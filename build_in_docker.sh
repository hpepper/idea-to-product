#!/bin/bash

# docker run -it --rm -v $(pwd)/sad2md:/home/builder/project -v $(pwd)/bin:/home/builder/bin rust:1.98 /home/builder/project/release.sh

# https://hub.docker.com/_/rust
docker run -it --rm --user "$(id -u)":"$(id -g)" -v $(pwd):/home/builder/project -v $(pwd)/bin:/home/builder/bin -w /home/builder rust:1.98 ./project/sad2md/release.sh

cp bin/sad2md $HOME/bin

