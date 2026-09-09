## ADDED Requirements

### Requirement: Auto-select device when none is connected

The system SHALL automatically select the first valid device and start log streaming when no device is currently selected and at least one valid device is available. A valid device is one whose adb state is `device` (authorized and online).

#### Scenario: Startup with no devices

- **WHEN** the application starts and no adb devices are available
- **THEN** no device is selected and log streaming remains disabled

#### Scenario: Startup with a device already connected

- **WHEN** the application starts and at least one valid device is available
- **THEN** the system automatically selects the first valid device and starts log streaming

#### Scenario: Device plugged in while idle

- **WHEN** no device is selected, log streaming is not active, and a valid device newly appears in the device list
- **THEN** the system automatically selects that device and starts log streaming

### Requirement: Preserve logs on disconnect

When the active device is unplugged or the logcat stream ends unexpectedly, the system SHALL stop streaming and enter a disconnected state without clearing the existing log buffer.

#### Scenario: Active device unplugged

- **WHEN** the user is streaming logs from a device and that device is unplugged or the stream fails
- **THEN** the system stops streaming, shows a disconnected state, and retains the previously received log entries in the buffer

### Requirement: Auto-reconnect on newly appeared device while disconnected

When log streaming is not active and a valid device newly appears in the device list, the system SHALL automatically select that newly appeared device and start log streaming.

#### Scenario: Replug same device after disconnect

- **WHEN** streaming was active, the device was unplugged causing a disconnected state, and the same device reappears as a valid entry in the device list
- **THEN** the system automatically reconnects to that device and resumes log streaming

#### Scenario: Replug different device after disconnect

- **WHEN** streaming was active, the device was unplugged causing a disconnected state, and a different valid device newly appears in the device list
- **THEN** the system automatically selects the newly appeared device and starts log streaming from it

### Requirement: Do not auto-switch while streaming on another device

When log streaming is active on a selected device, the system SHALL NOT automatically switch to another device merely because an additional device appears in the list.

#### Scenario: Second device plugged in while streaming

- **WHEN** the system is actively streaming from device A and device B newly appears in the device list
- **THEN** the system continues streaming from device A and does not change the selected device

### Requirement: Auto-fallback when active device disappears

When the currently selected device disappears from the device list while other valid devices remain, the system SHALL automatically select the first remaining valid device and start log streaming.

#### Scenario: Active device unplugged with another device present

- **WHEN** the system is streaming from device A, device A disappears from the device list, and device B remains as a valid device
- **THEN** the system automatically selects device B and starts log streaming from device B
