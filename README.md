# 6.824-MapReduce

An implementation of "6.824 Lab 1: MapReduce (2021)" in async Rust.

```bash
$ make APP=wc seq   # sequential impl for reference
$ make APP=wc dist  # distributed impl
$ make diff         # compare the outputs
```

You may write your own _App_  in a crate named `app-myapp` and specify it through the `APP=myapp` override in Makefile. Take [app-indexer](app-indexer) for an example.
