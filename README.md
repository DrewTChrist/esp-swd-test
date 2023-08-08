# esp32-swd-test

This is a test of flashing a Raspberry Pi Pico with an Adafruit Huzzah32. The 
esp32 does not natively support SWD. This is useful for boards like the Udoo 
Key which have both an rp2040 and an esp32, where the esp32 is connected to 
the SWD pins of the rp2040.
