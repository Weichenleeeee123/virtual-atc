#!/usr/bin/env python3
"""
MSFS SimConnect Bridge
连接 MSFS 并通过 UDP 发送飞行数据到 Virtual ATC
"""

import sys
import time
import json
import socket
from SimConnect import SimConnect, AircraftRequests, AircraftEvents

def main():
    print("Starting MSFS SimConnect bridge...")
    
    try:
        # 连接到 MSFS
        sm = SimConnect()
        aq = AircraftRequests(sm, _time=200)
        
        print("Connected to MSFS")
        
        # 创建 UDP socket
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        target = ("127.0.0.1", 49001)
        
        print("UDP socket created, sending to", target)
        
        # 主循环
        while True:
            try:
                # 读取飞行数据
                data = {
                    "callsign": aq.get("ATC_ID") or "",
                    "altitude": aq.get("INDICATED_ALTITUDE") or 0.0,
                    "speed": aq.get("AIRSPEED_INDICATED") or 0.0,
                    "heading": aq.get("PLANE_HEADING_DEGREES_MAGNETIC") or 0.0,
                    "vertical_speed": aq.get("VERTICAL_SPEED") or 0.0,
                    "latitude": aq.get("PLANE_LATITUDE") or 0.0,
                    "longitude": aq.get("PLANE_LONGITUDE") or 0.0,
                    "on_ground": bool(aq.get("SIM_ON_GROUND") or 0),
                }
                
                # 发送 JSON 数据
                json_data = json.dumps(data).encode('utf-8')
                sock.sendto(json_data, target)
                
                # 每秒更新 5 次
                time.sleep(0.2)
                
            except Exception as e:
                print(f"Error reading data: {e}", file=sys.stderr)
                time.sleep(1)
                
    except Exception as e:
        print(f"Failed to connect to MSFS: {e}", file=sys.stderr)
        print("Make sure MSFS is running and SimConnect is enabled", file=sys.stderr)
        sys.exit(1)
    finally:
        if 'sock' in locals():
            sock.close()
        if 'sm' in locals():
            sm.exit()

if __name__ == "__main__":
    main()
