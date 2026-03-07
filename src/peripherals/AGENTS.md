# AGENTS.md — src/peripherals

Hardware peripherals that extend the agent with physical capabilities. See `docs/hardware-peripherals-design.md` for full design.

## WHERE TO LOOK

- `traits.rs` — `Peripheral` trait definition
- `serial.rs` — STM32/ESP32/Arduino over USB CDC serial
- `rpi.rs` — Raspberry Pi GPIO (native, Linux-only, feature `peripheral-rpi`)
- `arduino_flash.rs` — Flash zeroclaw-arduino firmware via arduino-cli
- `nucleo_flash.rs` — Flash STM32 Nucleo firmware
- `uno_q_bridge.rs` — Arduino Uno Q network bridge transport
- `capabilities_tool.rs` — Query device capabilities over serial

## CONVENTIONS

**Peripheral trait:**
- Implement `Peripheral` for each board type (e.g., `SerialPeripheral`, `RpiGpioPeripheral`)
- `name()` must uniquely identify the instance (include index/serial for multiple same-type boards)
- `board_type()` returns a stable lowercase config key (e.g., `"nucleo-f401re"`, `"rpi-gpio"`)
- `tools()` returns `Vec<Box<dyn Tool>>` — each tool delegates to hardware (gpio_read, gpio_write, sensor_read)
- Implementations must be `Send + Sync` (accessed from multiple async tasks)

**Tool exposure pattern:**
- Tools are created in `tools()` and hold an `Arc<Transport>` or owned config
- Tool names are shared across peripherals (`gpio_read`, `gpio_write`) but descriptions should specify the board type
- Use `spawn_blocking` for blocking GPIO operations (rppal, sysfs)

**Serial security:**
- Path validation via `ALLOWED_PATH_PREFIXES` (see `serial.rs`):
  - `/dev/ttyACM*`, `/dev/ttyUSB*`, `/dev/tty.usbmodem*`, `/dev/cu.usbmodem*`, `/dev/tty.usbserial*`, `/dev/cu.usbserial*`, `COM*`
- Never allow arbitrary paths — deny-by-default for serial port access
- Timeout all serial operations (default 5s in `SERIAL_TIMEOUT_SECS`)

**Config registration:**
- Add board type to `PeripheralBoardConfig` in `src/config/schema.rs` if needed
- Register in `create_peripheral_tools()` in `mod.rs` — match on `board.transport` and `board.board`
- Feature gates: `#[cfg(feature = "hardware")]` for serial boards, `#[cfg(all(feature = "peripheral-rpi", target_os = "linux"))]` for RPi

**Firmware:**
- Firmware crates live in `firmware/` (zeroclaw-arduino, zeroclaw-esp32, zeroclaw-nucleo)
- Protocol: newline-delimited JSON (`{"id":"1","cmd":"gpio_write","args":{...}}`)
- Response: `{"id":"1","ok":true,"result":"..."}`
- Flash commands use external tools (arduino-cli, probe-rs, OpenOCD)

**Testing:**
- Unit tests can mock `PeripheralBoardConfig` without hardware
- Integration tests require physical hardware — gate with `#[ignore]` or feature flags
- Test config parsing, path validation, and tool schema generation without hardware

**Safety for actuators:**
- Restrict which pins are exposed (avoid power/reset pins)
- Document pin limitations in tool descriptions
- Prefer read-only operations by default; require explicit opt-in for write/control
- No secrets stored on peripheral firmware — host handles all auth

## ANTI-PATTERNS

- Do not bypass `ALLOWED_PATH_PREFIXES` validation for serial ports
- Do not expose GPIO pins that control power/reset/critical functions without explicit safeguards
- Do not store API keys or secrets in peripheral firmware
- Do not block async runtime with synchronous GPIO — use `spawn_blocking`
- Do not add new board types without updating config schema and factory wiring
- Do not skip timeout handling on serial operations
