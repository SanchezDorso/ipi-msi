target := riscv64gc-unknown-none-elf
mode := debug
kernel := target/$(target)/$(mode)/hs
bin := target/$(target)/$(mode)/hs.bin

objdump := rust-objdump --arch-name=riscv64
objcopy := rust-objcopy --binary-architecture=riscv64

.PHONY: kernel build clean qemu run env

run: run-inner

env:
	cargo install cargo-binutils
	rustup component add llvm-tools-preview rustfmt
	rustup target add $(target)

kernel:
	cargo build

$(bin): kernel
	$(objcopy) $(kernel) --strip-all -O binary $@

asm:
	$(objdump) -d $(kernel) | less

build: $(bin)
	make -C /home/dorso/msi/ipi_msi/opensbi-1.2 PLATFORM=generic \
    	FW_PAYLOAD=y \
    	FW_PAYLOAD_PATH=/home/dorso/msi/ipi_msi/hs/target/riscv64gc-unknown-none-elf/debug/hs.bin
clean:
	cargo clean
run-inner: build	
	qemu-system-riscv64 -nographic \
    -M virt,aclint=on,aia=aplic-imsic,aia-guests=1 \
	-cpu rv64 -m 3G -smp 2 \
    -bios /home/dorso/msi/ipi_msi/opensbi-1.2/build/platform/generic/firmware/fw_payload.elf

gdb:	
	qemu-system-riscv64 -nographic \
    -M virt,aclint=on,aia=aplic-imsic \
	-cpu rv64 -m 3G -smp 2 \
    -bios /home/dorso/msi/ipi_msi/opensbi-1.2/build/platform/generic/firmware/fw_payload.elf \
	-s -S
# mysbi:
# 	qemu-system-riscv64 -nographic \
#     -M virt,aclint=on,aia=aplic-imsic,aia-guests=1 \
# 	-cpu rv64 -m 3G -smp 2 \
#     -bios /home/dorso/msi/ipi_msi/sbi/target/riscv64gc-unknown-none-elf/debug/sbi \
#     -kernel /home/dorso/msi/ipi_msi/hs/target/riscv64gc-unknown-none-elf/debug/hs.bin 