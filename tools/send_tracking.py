#!/usr/bin/env python3
"""Send synthetic OpenSeeFace packets for an explicit, camera-free smoke test."""
import argparse
import math
import socket
import struct
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--port", type=int, default=11573)
parser.add_argument("--face-id", type=int, default=0)
parser.add_argument("--seconds", type=float, default=10)
args = parser.parse_args()
sender = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
print(f"SYNTHETIC OpenSeeFace data → 127.0.0.1:{args.port}; no camera is opened.")
start = time.monotonic()
while (elapsed := time.monotonic() - start) < args.seconds:
    packet = bytearray(1785)
    struct.pack_into("<di", packet, 0, time.time(), args.face_id)
    struct.pack_into("<ffff", packet, 12, 640, 480, 1, 1)
    packet[28] = 1
    struct.pack_into("<fff", packet, 49, 180 + 10 * math.sin(elapsed * .6),
                     25 * math.sin(elapsed), 90 + 8 * math.sin(elapsed * .8))
    struct.pack_into("<f", packet, 1729 + 12 * 4, max(0, math.sin(elapsed * 3)) * .8)
    sender.sendto(packet, ("127.0.0.1", args.port))
    time.sleep(1 / 60)
