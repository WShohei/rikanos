TARGET = libusb.a
OBJS = lib.o \
       logger.o \
       libcxx_support.o \
       newlib_support.o \
       pci.o \
			 usb-cpp/memory.o \
       usb-cpp/device.o \
       usb-cpp/xhci/ring.o \
       usb-cpp/xhci/trb.o \
       usb-cpp/xhci/xhci.o \
       usb-cpp/xhci/port.o \
       usb-cpp/xhci/device.o \
       usb-cpp/xhci/devmgr.o  \
       usb-cpp/xhci/registers.o \
       usb-cpp/classdriver/base.o \
       usb-cpp/classdriver/hid.o \
       usb-cpp/classdriver/keyboard.o \
       usb-cpp/classdriver/mouse.o

DEPENDS = $(join $(dir $(OBJS)),$(addprefix .,$(notdir $(OBJS:.o=.d))))

CPPFLAGS += -I.
CFLAGS   += -O2 -Wall -g --target=x86_64-elf -ffreestanding -mno-red-zone
CXXFLAGS += -O2 -Wall -g --target=x86_64-elf -ffreestanding -mno-red-zone \
            -fno-exceptions -fno-rtti -std=c++17 -I/home/shohei/osbook/devenv/x86_64-elf/include/c++/v1

.PHONY: all
all: $(TARGET)

.PHONY: clean
clean:
	find . -name "*.o" | xargs rm
	find . -name "*.d" | xargs rm -rf
	rm libusb.a

libusb.a: $(OBJS) Makefile
	ar rcs $@ $(OBJS)

%.o: %.cpp Makefile
	clang++ $(CPPFLAGS) $(CXXFLAGS) -c $< -o $@

.%.d: %.cpp
	clang++ $(CPPFLAGS) $(CXXFLAGS) -MM $< > $@
	$(eval OBJ = $(<:.cpp=.o))
	sed --in-place 's|$(notdir $(OBJ))|$(OBJ)|' $@

%.o: %.c Makefile
	clang $(CPPFLAGS) $(CFLAGS) -c $< -o $@

.%.d: %.c
	clang $(CPPFLAGS) $(CFLAGS) -MM $< > $@
	$(eval OBJ = $(<:.c=.o))
	sed --in-place 's|$(notdir $(OBJ))|$(OBJ)|' $@

.PHONY: depends
depends:
	$(MAKE) $(DEPENDS)

-include $(DEPENDS)
