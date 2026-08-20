#!/bin/bash

docker run -it --rm -v $(pwd)/sad2md:/home/builder/project -v $(pwd)/bin:/home/builder/bin rusty:1.85.0-2 ./project/release.sh
