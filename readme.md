# Example Benchmarking Code for DriveShaft

Alter the existing benchmarks under the benches folder for your workload or add a new benchmark altogether.

To run your benchmark use:

```bash
cargo bench --bench <your_benchmark_file_without_extension>
```

You may not see much of a difference between spawn_blocking and driveshaft at first. In order to see the difference you need to simulate a heavy task load. You can do this by decreasing the runtime threads and/or increasing the number of tasks running at once.

You may run into issues with running the benches involving I/O. These rely on RocksDB. You will need to install clang and llvm for them to work.
