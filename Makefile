MACOS_SDK_V6_ROOT = "/Users/andrey/Downloads/arm-unknown-linux-gnueabi"
MACOS_SDK_V6_SYSROOT=$(MACOS_SDK_V6_ROOT)/arm-unknown-linux-gnueabi/sysroot
MACOS_SDK_V6_LINKER=$(MACOS_SDK_V6_ROOT)/bin/arm-unknown-linux-gnueabi-gcc

gen_v6:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$(MACOS_SDK_V6_SYSROOT)/ -I$(MACOS_SDK_V6_SYSROOT)/usr/include/freetype2" \
	cargo build --features sdk_v6

publish:
	BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$(MACOS_SDK_V6_SYSROOT)/ -I$(MACOS_SDK_V6_SYSROOT)/usr/include/freetype2" \
	cargo publish --features sdk_v6