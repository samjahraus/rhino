use std::{thread, time};

use sysinfo::{System, SystemExt, CpuExt, ComponentExt, DiskExt};

use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::{Nvml};

//Windows Junk
extern crate kernel32;
extern crate winapi;

use winapi::{HANDLE};
use winapi::wincon::CONSOLE_SCREEN_BUFFER_INFO;
use winapi::wincon::COORD;
use winapi::wincon::SMALL_RECT;
use winapi::WORD;
use winapi::DWORD;

static mut CONSOLE_HANDLE: Option<HANDLE> = None;

fn main() -> Result<(), NvmlError>{

    println!("Initializing Rhino...");

    let nvml = Nvml::init()?;

    // Cool Initialization
    for n in 1..7 {
        println!("...");
        thread::sleep(time::Duration::from_millis(125));
    }
    println!("");

    let cuda_version = nvml.sys_cuda_driver_version()?;

    let device = nvml.device_by_index(0)?;

    let gpu_name = device.name()?;
    let gpu_architecture = device.architecture()?;

    let mut sys = System::new_all();
    sys.refresh_system();

    println!("System Info");
    println!("----------------------------------------------------");

    loop { 
        let mut cpu_utlization: f32 = 0.0;

        let mut sys_data: SystemData = SystemData::new();
        sys_data.system_host_name = sys.host_name();
        sys_data.system_os_name = sys.name();
        sys_data.system_os_version = sys.os_version();
    
        println!("System Name: {}", sys_data.system_host_name.unwrap());
        println!("OS: {} Version: {}", sys_data.system_os_name.unwrap(), sys_data.system_os_version.unwrap());
        println!("CPU Name: Default CPU Name");
        println!("Available CPU Cores: {}", sys.cpus().len());
        println!("GPU Name: {} ({})", gpu_name, gpu_architecture);
        println!("Cuda Version: {}", cuda_version);
        println!("");
        println!("CPU Info");
        println!("----------------------------------------------------");

        sys.refresh_cpu();
        sys.refresh_components();

        for cpu in sys.cpus() {
            //println!("{}: Usuage: {}", cpu.name(), cpu.cpu_usage());
            cpu_utlization += cpu.cpu_usage();
        }

        cpu_utlization /= sys.cpus().len() as f32;

        println!("CPU Utilization: {:.1}%", cpu_utlization);

        if sys.components().len() == 0 {
            println!("CPU Temperature(°C): [Requires Admin Privileges]");
        } else {
            for component in sys.components() {
                println!("CPU Temperature(°C): {:.1}°C", component.temperature());
            }
        }

        println!("");
        println!("GPU Info");
        println!("----------------------------------------------------");
        
        let gpu_utilization = device.utilization_rates()?.gpu;
        let gpu_temp = device.temperature(TemperatureSensor::Gpu)?;
        let gpu_clock = device.clock_info(Clock::Graphics)?;
        let gpu_mem_info = device.memory_info()?;
        let gpu_mem_clock = device.clock_info(Clock::Memory)?;

        let gpu_mem_total: f64 = gpu_mem_info.total as f64 / 1000000000.0;
        let gpu_mem_used: f64 = gpu_mem_info.used as f64 / 1000000000.0;

        println!("GPU Utilization: {:.1}%", gpu_utilization);
        println!("GPU Temperature(°C): {:.1}°C", gpu_temp);
        println!("GPU Clock Speed: {:.1}MHz", gpu_clock);
        println!("VRAM:{:.2}G/{:.2}G, {:.1}%", gpu_mem_used, gpu_mem_total, (gpu_mem_used / gpu_mem_total) * 100.0);
        println!("VRAM Clock Speed: {:.1}MHz", gpu_mem_clock);

        println!("");
        println!("Memory Info");
        println!("----------------------------------------------------");

        sys.refresh_memory();

        let total_memory: f64 = sys.total_memory()  as f64 / 1000000000.0;
        let used_memory: f64 = sys.used_memory() as f64 / 1000000000.0;

        println!("RAM:{:.2}G/{:.2}G, {:.1}%", used_memory, total_memory, (used_memory / total_memory) * 100.0);

        println!("");
        println!("Disk Info");
        println!("----------------------------------------------------");

        sys.refresh_disks();

        for disk in sys.disks() {

            let mut disk_name: &str = disk.name().to_str().unwrap();
            let disk_type = disk.type_();

            if disk_name.len() == 0 {
                disk_name = "N/A";
            }

            let disk_total_space = disk.total_space() as f64 / 1000000000.0;
            let disk_used_space = disk_total_space - (disk.available_space() as f64 / 1000000000.0);



            println!("({:?}) Disk Name: {}, {:.2}G/{:.2}G, {:.1}%", disk_type, disk_name, disk_used_space, disk_total_space, (disk_used_space / disk_total_space) * 100.0);
        }

        thread::sleep(time::Duration::from_millis(1000));
        clear();
    }
}

struct SystemData {
    system_host_name: Option<String>,
    system_os_name: Option<String>,
    system_os_version: Option<String>,
}

impl Default for SystemData {
    fn default() -> Self {
        Self {
            system_host_name:       Some(String::from("Default_System_Name")),
            system_os_name:         Some(String::from("Default_OS_Name")),
            system_os_version:      Some(String::from("XXXX")),
        }
    }
}

impl SystemData {
    pub fn new() -> Self {
        Default::default()
    }
}

fn get_output_handle() -> HANDLE {
    unsafe {
        if let Some(handle) = CONSOLE_HANDLE {
            return handle;
        } else {
            let handle = kernel32::GetStdHandle(winapi::STD_OUTPUT_HANDLE);
            CONSOLE_HANDLE = Some(handle);
            return handle;
        }
    }
}

fn get_buffer_info() -> winapi::CONSOLE_SCREEN_BUFFER_INFO {
    let handle = get_output_handle();
    if handle == winapi::INVALID_HANDLE_VALUE {
        panic!("NoConsole")
    }
    let mut buffer = CONSOLE_SCREEN_BUFFER_INFO {
        dwSize: COORD { X: 0, Y: 0 },
        dwCursorPosition: COORD { X: 0, Y: 0 },
        wAttributes: 0 as WORD,
        srWindow: SMALL_RECT {
            Left: 0,
            Top: 0,
            Right: 0,
            Bottom: 0,
        },
        dwMaximumWindowSize: COORD { X: 0, Y: 0 },
    };
    unsafe {
        kernel32::GetConsoleScreenBufferInfo(handle, &mut buffer);
    }
    buffer
}

fn clear() {
    let handle = get_output_handle();
    if handle == winapi::INVALID_HANDLE_VALUE {
        panic!("NoConsole")
    }

    let screen_buffer = get_buffer_info();
    let console_size: DWORD = screen_buffer.dwSize.X as u32 * screen_buffer.dwSize.Y as u32;
    let coord_screen = COORD { X: 0, Y: 0 };

    let mut amount_chart_written: DWORD = 0;
    unsafe {
        kernel32::FillConsoleOutputCharacterW(
            handle,
            32 as winapi::WCHAR,
            console_size,
            coord_screen,
            &mut amount_chart_written,
        );
    }
    set_cursor_possition(0, 0);
}

fn set_cursor_possition(y: i16, x: i16) {
    let handle = get_output_handle();
    if handle == winapi::INVALID_HANDLE_VALUE {
        panic!("NoConsole")
    }
    unsafe {
        kernel32::SetConsoleCursorPosition(handle, COORD { X: x, Y: y });
    }
}
