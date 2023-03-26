wmic cpu list brief /format:list  > log_system.txt
wmic path Win32_VideoController get Name,AdapterRAM,DriverVersion,VideoModeDescription /format:list >> log_system.txt
SET RUST_LOG=info
SET VK_LOADER_DEBUG=all
farmisto.exe > log.txt 2>&1
pause