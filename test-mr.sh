#!/usr/bin/env bash

#
# basic map-reduce test
#

MAKE="make -C .."

# run the test in a fresh sub-directory.
rm -rf mr-tmp
mkdir mr-tmp || exit 1
cd mr-tmp || exit 1
rm -f mr-*

# killall dead processes
killall coordinator
killall worker
killall sequential

# make sure software is freshly built.
$MAKE build

failed_any=0

#########################################################
# first word-count

# generate the correct output
$MAKE APP=wc seq || exit 1
sort ../mr-out-0 > mr-correct-wc.txt
$MAKE clean

echo '***' Starting wc test.

timeout -k 2s 180s $MAKE dist-coordinator &
pid=$!

# give the coordinator time to create the sockets.
sleep 1

# start multiple workers.
timeout -k 2s 180s $MAKE APP=wc dist-worker &
timeout -k 2s 180s $MAKE APP=wc dist-worker &
timeout -k 2s 180s $MAKE APP=wc dist-worker &

# wait for the coordinator to exit.
wait $pid

# since workers are required to exit when a job is completely finished,
# and not before, that means the job has finished.
sort ../out/mr-out* | grep . > mr-wc-all
if cmp mr-wc-all mr-correct-wc.txt
then
  echo '---' wc test: PASS
else
  echo '---' wc output is not the same as mr-correct-wc.txt
  echo '---' wc test: FAIL
  failed_any=1
fi

# wait for remaining workers and coordinator to exit.
wait

#########################################################
# now indexer
$MAKE clean

# generate the correct output
$MAKE APP=indexer seq || exit 1
sort ../mr-out-0 > mr-correct-indexer.txt
$MAKE clean

echo '***' Starting indexer test.

timeout -k 2s 180s $MAKE dist-coordinator &
sleep 1

# start multiple workers
timeout -k 2s 180s $MAKE APP=indexer dist-worker &
timeout -k 2s 180s $MAKE APP=indexer dist-worker

sort ../out/mr-out* | grep . > mr-indexer-all
if cmp mr-indexer-all mr-correct-indexer.txt
then
  echo '---' indexer test: PASS
else
  echo '---' indexer output is not the same as mr-correct-indexer.txt
  echo '---' indexer test: FAIL
  failed_any=1
fi

wait

#########################################################
echo '***' Starting job count test.

$MAKE clean

timeout -k 2s 180s $MAKE dist-coordinator &
sleep 1

timeout -k 2s 180s $MAKE APP=jobcount dist-worker &
timeout -k 2s 180s $MAKE APP=jobcount dist-worker
timeout -k 2s 180s $MAKE APP=jobcount dist-worker || true &
timeout -k 2s 180s $MAKE APP=jobcount dist-worker || true

NT=`cat ../out/mr-out* | awk '{print $2}'`
if [ "$NT" -ne "8" ]
then
  echo '---' map jobs ran incorrect number of times "($NT != 8)"
  echo '---' job count test: FAIL
  failed_any=1
else
  echo '---' job count test: PASS
fi

wait

#########################################################
# test whether any worker or coordinator exits before the
# task has completed (i.e., all output files have been finalized)
$MAKE clean

echo '***' Starting early exit test.

timeout -k 2s 180s $MAKE dist-coordinator &

# give the coordinator time to create the sockets.
sleep 1

# start multiple workers.
timeout -k 2s 180s $MAKE APP=early_exit dist-worker &
pid=$!
timeout -k 2s 180s $MAKE APP=early_exit dist-worker &
timeout -k 2s 180s $MAKE APP=early_exit dist-worker &

# wait for any of the coord or workers to exit
# `jobs` ensures that any completed old processes from other tests
# are not waited upon
# jobs &> /dev/null
wait $pid

# a process has exited. this means that the output should be finalized
# otherwise, either a worker or the coordinator exited early
sort ../out/mr-out* | grep . > mr-wc-all-initial

# wait for remaining workers and coordinator to exit.
wait

# compare initial and final outputs
sort ../out/mr-out* | grep . > mr-wc-all-final
if cmp mr-wc-all-final mr-wc-all-initial
then
  echo '---' early exit test: PASS
else
  echo '---' output changed after first worker exited
  echo '---' early exit test: FAIL
  failed_any=1
fi
$MAKE clean

#########################################################
echo '***' Starting crash test.

# generate the correct output
$MAKE APP=crash seq || exit 1
sort ../mr-out-0 > mr-correct-crash.txt
$MAKE clean

rm -f mr-done
($MAKE APP=crash dist-coordinator ; touch mr-done ) &
sleep 1

# start multiple workers
CRASH=1 timeout -k 2s 180s $MAKE APP=crash dist-worker &

( while [ ! -f mr-done ]
  do
    CRASH=1 timeout -k 2s 180s $MAKE APP=crash dist-worker || true
    sleep 1
  done ) &

( while [ ! -f mr-done ]
  do
    CRASH=1 timeout -k 2s 180s $MAKE APP=crash dist-worker  || true
    sleep 1
  done ) &

while [ ! -f mr-done ]
do
  CRASH=1 timeout -k 2s 180s $MAKE APP=crash dist-worker || true
  sleep 1
done

wait

sort ../out/mr-out* | grep . > mr-crash-all
if cmp mr-crash-all mr-correct-crash.txt
then
  echo '---' crash test: PASS
else
  echo '---' crash output is not the same as mr-correct-crash.txt
  echo '---' crash test: FAIL
  failed_any=1
fi

#########################################################
if [ $failed_any -eq 0 ]; then
    echo '***' PASSED ALL TESTS
else
    echo '***' FAILED SOME TESTS
    exit 1
fi
