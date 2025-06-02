#!/bin/sh
echo "loading Loader_Worldgen.java"
rg "WorldgenOresVanilla" Loader_Worldgen.java | sed -n 's/.*\.\([^"]*\)".*/\1/p' > oresVanilla.txt
rg "WorldgenOresBedrock" Loader_Worldgen.java | sed -n 's/.*\.\([^"]*\)".*/\1/p' > oresBedrock.txt
rg "WorldgenOresSmall" Loader_Worldgen.java | sed -n 's/.*\.\([^"]*\)".*/\1/p' > oresSmall.txt
rg "WorldgenOresLarge" Loader_Worldgen.java | sed -n 's/.*\.\([^"]*\)".*/\1/p' > oresLarge.txt
echo "Extraction Complete"
