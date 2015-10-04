#!/bin/bash

cd $(dirname `readlink -f "$0"`)
mkdir -p ../solutions

for i in ../levels/[0-9]*.xml; do
  dest=${i//levels/solutions}
  dest=${dest//.xml/.txt}
  [ -e $dest ] && continue
  echo $i
  ../transmission_solver $i > $dest
done
