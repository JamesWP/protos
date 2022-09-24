#!/bin/bash

cat input.hex | xxd -r -p | nc localhost 20345