#!/bin/bash

for i in ../levels/[0-9]*.xml; do
  dest=${i//.xml/.txt}
  [ -e $dest ] && continue
  echo $i
  if ! ruby parse_cutted_xml.rb $i > $dest; then
    rm -f $dest
    exit 1
  fi
done
