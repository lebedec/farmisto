use ash::vk;

#[derive(Clone, Debug)]
pub struct Screen {
    pub surface: vk::SurfaceKHR,
    pub format: vk::SurfaceFormatKHR,
    pub resolution: vk::Extent2D,
    pub scissors: Vec<vk::Rect2D>,
    pub viewports: Vec<vk::Viewport>,
}

impl Screen {
    pub fn new(
        surface: vk::SurfaceKHR,
        format: vk::SurfaceFormatKHR,
        resolution: vk::Extent2D,
    ) -> Self {
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
            format,
            resolution,
            scissors,
            viewports,
        }
    }

    #[inline]
    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    #[inline]
    pub fn format(&self) -> vk::Format {
        self.format.format
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
