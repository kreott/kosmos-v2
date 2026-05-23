BINS = ls cat mkdir rmdir rm
BIN_DIR = target/x86_64-kosmos/debug
MNT = /tmp/mnt

.PHONY: all build clean move

all: build move

build:
	cargo build
	cargo build -p kosmos-std
	cargo build -p kosmos-bin

move:
	sudo umount $(MNT) || true
	sudo mount --mkdir qemu-fat32.img $(MNT)
	sudo mkdir -p $(MNT)/usr/bin
	sudo rm -rf $(MNT)/usr/bin/*
	$(foreach bin,$(BINS),sudo cp $(BIN_DIR)/$(bin) $(MNT)/usr/bin/;)
	sudo umount $(MNT)

clean:
	cargo clean
	sudo umount $(MNT) || true
	sudo mount --mkdir qemu-fat32.img $(MNT)
	sudo rm -rf $(MNT)/usr/bin/*
	sudo umount $(MNT)