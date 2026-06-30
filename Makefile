.PHONY: generate

generate:
	rm -rf sdk
	mkdir -p sdk
	cd sudoku-bindings && \
		echo "$$PWD" && \
		IPHONEOS_DEPLOYMENT_TARGET=13.0 \
		cargo swift --accept-all package \
			-p ios \
			--release && \
		mv SudokuBindings ../sdk/