use ash::extensions::khr::Surface;
use ash::vk;
use log::info;

#[derive(Clone)]
pub struct Screen {
    surface: vk::SurfaceKHR,
    pub surface_loader: Surface,
    physical_device: vk::PhysicalDevice,
    format: vk::SurfaceFormatKHR,
    resolution: vk::Extent2D,
    pub scissors: Vec<vk::Rect2D>,
    pub viewports: Vec<vk::Viewport>,
}

impl Screen {
    pub fn new(
        surface: vk::SurfaceKHR,
        surface_loader: Surface,
        physical_device: vk::PhysicalDevice,
    ) -> Self {
        let format = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()[0]
        };
        let caps = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap()
        };
        let resolution = match caps.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: 800,
                height: 600,
            },
            _ => caps.current_extent,
        };

        let viewports = vec![vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: resolution.width as f32,
            height: resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = vec![resolution.into()];
        Self {
            surface,
            surface_loader,
            physical_device,
            format,
            resolution,
            scissors,
            viewports,
        }
    }

    pub fn resize(&mut self) {
        let caps = unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(self.physical_device, self.surface)
                .unwrap()
        };
        let resolution = match caps.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: 800,
                height: 600,
            },
            _ => caps.current_extent,
        };
        info!(
            "Recalculates viewports from {:?} to {:?}",
            self.resolution, resolution
        );
        self.resolution = resolution;
        self.viewports = vec![vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: resolution.width as f32,
            height: resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        self.scissors = vec![resolution.into()];
    }

    #[inline]
    pub fn resolution(&self) -> vk::Extent2D {
        self.resolution
    }

    #[inline]
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    #[inline]
    pub fn present_modes(&self) -> Vec<vk::PresentModeKHR> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(self.physical_device, self.surface)
                .unwrap()
        }
    }

    #[inline]
    pub fn get_capabilities(&self) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(self.physical_device, self.surface)
                .unwrap()
        }
    }

    #[inline]
    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    #[inline]
    pub fn format(&self) -> vk::SurfaceFormatKHR {
        self.format
    }

    #[inline]
    pub fn size(&self) -> [u32; 2] {
        [self.resolution.width, self.resolution.height]
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.resolution.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.resolution.height
    }
}
