EMFLAGS= --bind --memory-init-file 0 --closure 1 -O3 --post-js post.js -s INLINING_LIMIT=50 -s ALLOW_MEMORY_GROWTH=1 -s MODULARIZE=0 -s INVOKE_RUN=0 -s NO_EXIT_RUNTIME=1 -s TOTAL_STACK=32768

all: transmission_solver.js index.html

transmission_solver.js: ../transmission_solver.cc post.js
	emcc -Wall -std=c++11 -O2 $(EMFLAGS) -s EXPORTED_FUNCTIONS="['_solveLevelXML', '_printConnectableFromLevelXML']" $< -o $@

index.html: index.slim
	slimrb $^ > $@
