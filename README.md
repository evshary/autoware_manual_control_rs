# autoware_manual_control_rs

Control the vehicle in Autoware with native Zenoh API.

Note that the program should run with zenoh-bridge-dds.

# Build

* Build autoware_manual_control_rs

```shell
cd ~
git clone https://github.com/evshary/autoware_manual_control_rs.git
cd autoware_manual_control_rs
cargo build --release
```

* Build zenoh-bridge-dds

```shell
git clone https://github.com/eclipse-zenoh/zenoh-plugin-dds.git
cd zenoh-plugin-dds
# Note that the zenoh version of autoware_manual_control_rs and zenoh-bridge-dds should match
git checkout 0.7.0-rc
cargo build --release -p zenoh-bridge-dds
```

* Create `myconfig.json` under zenoh-bridge-dds
  - We want to set the topic filter

```json
{
    "plugins": {
        "dds": {
            "allow": "/external/selected/control_cmd|/external/selected/gear_cmd|/control/gate_mode_cmd|/api/autoware/set/engage|/control/current_gate_mode|/api/autoware/get/engage|/vehicle/status/velocity_status|/vehicle/status/gear_status"
        }
    }
}
```

# Run

* 1st terminal: Run Autoware1

```shell
rocker --network host --privileged --x11 --user --volume $HOME/autoware_map -- ghcr.io/autowarefoundation/autoware-universe:latest-prebuilt bash
# Inside the docker
ROS_DOMAIN_ID=1 ros2 launch autoware_launch planning_simulator.launch.xml map_path:=$HOME/autoware_map/sample-map-planning vehicle_model:=sample_vehicle sensor_model:=sample_sensor_kit
```

* 2nd terminal: Run Autoware2

```shell
rocker --network host --privileged --x11 --user --volume $HOME/autoware_map -- ghcr.io/autowarefoundation/autoware-universe:latest-prebuilt bash
# Inside the docker
ROS_DOMAIN_ID=2 ros2 launch autoware_launch planning_simulator.launch.xml map_path:=$HOME/autoware_map/sample-map-planning vehicle_model:=sample_vehicle sensor_model:=sample_sensor_kit
```

* 3rd terminal: Run zenoh-bridge-dds for Autoware1

```shell
cd ~/zenoh-bridge-dds
./target/release/zenoh-bridge-dds -c myconfig.json5 -s "v1" -d 1
```

* 4th terminal: Run zenoh-bridge-dds for Autoware2

```shell
cd ~/zenoh-bridge-dds
./target/release/zenoh-bridge-dds -c myconfig.json5 -s "v2" -d 2
```

* 5th terminal: Run manual_control for two vehicles

```shell
cd ~/autoware_manual_control_rs
./target/release/autoware_manual_control -s "*"
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
