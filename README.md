# autoware_manual_control_rs

Control the vehicle in Autoware with native Zenoh API.

Note that the program should run with zenoh-bridge-dds / zenoh-bridge-ros2dds.

# Quick Demo

If you want to try the autoware_manual_control_rs within containers, try the following links

* [autoware_teleop_with_bridge_dds](https://github.com/evshary/zenoh_demo_docker_env/tree/main/autoware_teleop_with_bridge_dds)
* [autoware_teleop_with_bridge_ros2dds](https://github.com/evshary/zenoh_demo_docker_env/tree/main/autoware_teleop_with_bridge_ros2dds)

# Build

* Build autoware_manual_control_rs

```shell
cd ~
git clone https://github.com/evshary/autoware_manual_control_rs.git
cd autoware_manual_control_rs
cargo build --release
```

* Option 1: Use zenoh-bridge-dds

```shell
# Build zenoh-bridge-dds
git clone https://github.com/eclipse-zenoh/zenoh-plugin-dds.git
cd zenoh-plugin-dds
# Note that the zenoh version of autoware_manual_control_rs and zenoh-bridge-dds should match
cargo build --release -p zenoh-bridge-dds
cd ..
```

* Option 2: Use zenoh-bridge-ros2dds

```shell
# Build zenoh-bridge-ros2dds
git clone https://github.com/eclipse-zenoh/zenoh-plugin-ros2dds.git
cd zenoh-plugin-ros2dds
# Note that the zenoh version of autoware_manual_control_rs and zenoh-bridge-ros2dds should match
cargo build --release -p zenoh-bridge-ros2dds
cd ..
```

# Run

* 1st terminal: Run Autoware1

```shell
# Get the map
mkdir ~/autoware_map
gdown -O ~/autoware_map/ 'https://docs.google.com/uc?export=download&id=1499_nsbUbIeturZaDj7jhUownh5fvXHd'
unzip -d ~/autoware_map ~/autoware_map/sample-map-planning.zip
# Run Autoware docker
rocker --network host --privileged --x11 --user --volume $HOME/autoware_map -- ghcr.io/autowarefoundation/autoware-universe:galactic-20221115-prebuilt-amd64 bash
# Inside the docker
ROS_DOMAIN_ID=1 ros2 launch autoware_launch planning_simulator.launch.xml map_path:=$HOME/autoware_map/sample-map-planning vehicle_model:=sample_vehicle sensor_model:=sample_sensor_kit
```

* 2nd terminal: Run Autoware2

```shell
# Run Autoware docker
rocker --network host --privileged --x11 --user --volume $HOME/autoware_map -- ghcr.io/autowarefoundation/autoware-universe:galactic-20221115-prebuilt-amd64 bash
# Inside the docker
ROS_DOMAIN_ID=2 ros2 launch autoware_launch planning_simulator.launch.xml map_path:=$HOME/autoware_map/sample-map-planning vehicle_model:=sample_vehicle sensor_model:=sample_sensor_kit
```

* Option1: Use zenoh-bridge-dds

```shell
# terminal 3
./zenoh-plugin-dds/target/release/zenoh-bridge-dds -c zenoh-bridge-dds.json5 -s "v1" -d 1
# terminal 4
./zenoh-plugin-dds/target/release/zenoh-bridge-dds -c zenoh-bridge-dds.json5 -s "v2" -d 2
# terminal 5
./target/release/autoware_manual_control -p "*" -m dds
```

* Option2: Use zenoh-bridge-ros2dds

```shell
# terminal 3
./zenoh-plugin-ros2dds/target/release/zenoh-bridge-ros2dds -c zenoh-bridge-ros2dds.json5 -n "/v1" -d 1
# terminal 4
./zenoh-plugin-ros2dds/target/release/zenoh-bridge-ros2dds -c zenoh-bridge-ros2dds.json5 -n "/v2" -d 2
# terminal 5
./target/release/autoware_manual_control -p "*" -m ros2
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
