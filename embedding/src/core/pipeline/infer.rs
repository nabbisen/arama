use candle_core::Device;

pub mod clip;

fn device() -> Device {
    Device::new_cuda(0).unwrap_or(Device::new_metal(0).unwrap_or(Device::Cpu))
}
