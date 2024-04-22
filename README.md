# bambulab-cloud

> An unofficial API client for Bambu Lab's Cloud API.

[![crates.io](https://img.shields.io/crates/v/bambulab-cloud.svg)](https://crates.io/crates/bambulab-cloud)
[![download count badge](https://img.shields.io/crates/d/bambulab-cloud.svg)](https://crates.io/crates/bambulab-cloud)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/bambulab-cloud)

## Usage

```rust
let client = bambulab_cloud::Client::login(bambulab_cloud::Region::Europe, "email@example.com", "password").await?;

let tasks = client.get_tasks().await?;

dbg!(tasks);
// [src/main.rs:6] tasks = [
//   Task {
//      id: 67318297,
//      length: 2783,
//      weight: 81.66,
//      cost_time: 6541s,
//      cover: Url { ... },
//      end_time: 2024-04-06T01:51:58Z,
//      start_time: 2024-04-05T23:56:48Z,
//      design_title: "Cursed Benchys #1",
//      title: "0.24mm layer, 3 walls, 30% infill",
//      ...
//   },
// ]
```

Refer to the [documentation on docs.rs](https://docs.rs/bambulab-cloud) for detailed usage instructions.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
