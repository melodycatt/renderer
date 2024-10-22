use winit::{
    event::{
        WindowEvent,
        KeyEvent,
        ElementState
    },
    keyboard::{
        KeyCode,
        PhysicalKey
    }
};
use cgmath::{num_traits::Pow, Vector3};

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    pub rotation: Vector3<f32>
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
 


pub struct CameraController {
    pub speed: f32,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyE => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyQ => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }
        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so 
            // that it doesn't change. The eye, therefore, still 
            // lies on the circle made by the target and eye.
            //camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
            camera.rotation.y += self.speed;
        }
        if self.is_left_pressed {
            //camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
            camera.rotation.y -= self.speed;
        }
        if self.is_down_pressed {
            //camera.eye = camera.target - (forward + camera.up * self.speed).normalize() * forward_mag;
            camera.rotation.x += self.speed;
        }     
        if self.is_up_pressed {
            //camera.eye = camera.target - (forward - camera.up * self.speed).normalize() * forward_mag;
            camera.rotation.x -= self.speed;
        }

        let rotmod = Vector3::new(
            camera.rotation.x % (2.0 * std::f32::consts::PI),
            camera.rotation.y % (2.0 * std::f32::consts::PI),
            camera.rotation.z % (2.0 * std::f32::consts::PI),
        );

        let forward = Vector3::new(
            forward_mag * camera.rotation.x.cos() * camera.rotation.y.cos(),
            forward_mag * camera.rotation.x.sin(),
            forward_mag * camera.rotation.x.cos() * camera.rotation.y.sin(),
        );
        let forward_norm = forward.normalize();
        camera.eye = camera.target - forward;
        let right = forward_norm.cross(Vector3::new(
            0.0,
            1.0 * if rotmod.x.abs() > std::f32::consts::PI / 2.0 && rotmod.x.abs() > 3.0 * std::f32::consts::PI / 2.0 { -1.0 } else { 1.0 },
            0.0,
        )).normalize();
        //let right = Vector3::new()
        
        camera.up = forward_norm.cross(right);

        //let to_target = camera.eye - camera.target;
        //camera.rotation = Vector3::new(to_target.x.atan2(to_target.z) * 180.0 / std::f32::consts::PI + 180.0, to_target.y.atan2((to_target.x.powi(2) + to_target.z.powi(2)).sqrt()) * 180.0 / std::f32::consts::PI + 180.0, camera.rotation.z);
        println!("UP: {:#?} \nFORWARD: {:#?} \nRIGHT: {:#?} \nROT: {:#?}", camera.up, forward_norm, right, camera.rotation);
    }
}
