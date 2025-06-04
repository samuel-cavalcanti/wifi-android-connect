#!/bin/bash

cargo b --release -p=wifi-android-connect
cp ./target/release/wifi-android-connect ./lua/
