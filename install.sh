#!/bin/sh

for folder in *
do
  [ -d "$folder" ] && cargo install --path "$folder"
done
