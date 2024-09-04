#!/bin/bash

cargo b --release -p=wifi-android-connect-nvim 
cp ./target/release/libwifi_android_connect_nvim.so ./lua/
