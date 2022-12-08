# autoware_manual_control_rs

Control the vehicle in Autoware with native Zenoh API.

Note that the program should run with zenoh-bridge-dds.

# Build

* Build

```shell
git clone https://github.com/evshary/autoware_manual_control_rs.git
cd autoware_manual_control_rs
cargo build
```

* Run

```shell
cargo run
```

# Usage

1. Toggle to external mode
2. Set Gear Type to Drive
3. Adjust speed and steering angle
4. Enjoy driving :-)

```
------------------------------------
| Different Mode:                  |
|   z: Toggle auto & external mode |
|   x: Gear Type => Drive          |
|   c: Gear Type => Reverse        |
|   v: Gear Type => Park           |
|   s: View current mode           |
| Speed:                           |
|   u: Increase speed              |
|   i: Set speed to 0              |
|   o: Decrease speed              |
| Steering Angle                   |
|   j: Left turn                   |
|   k: Set angle to 0              |
|   l: Right turn                  |
------------------------------------
```

# Reference

* [autoware_manual_control](https://github.com/evshary/autoware_manual_control): control Autoware with ROS 2 topic directly.
