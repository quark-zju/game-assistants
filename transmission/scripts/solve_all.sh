#!/bin/bash

mkdir -p ../solutions

for i in ../levels/[0-9]*.txt; do
  dest=${i//levels/solutions}
  [ -e $dest ] && continue
  echo $i
  ../transmission_solver < $i > $dest
done
