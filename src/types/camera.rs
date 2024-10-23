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
use cgmath::{Vector3, InnerSpace};
use std::f32::consts::PI;

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

    // If you're wondering, we're not using a Quaternion because that adds an extra level of complication
    // when we don't need to worry about gimbal lock - all rotations will be manual, so it won't affect any calculations
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
    pub is_zcw_pressed: bool,
    pub is_zccw_pressed: bool,
    pub is_debug_pressed: bool,
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
            is_zcw_pressed: false,
            is_zccw_pressed: false,
            is_debug_pressed: false,
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
                    KeyCode::KeyC => {
                        self.is_zcw_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyZ => {
                        self.is_zccw_pressed = is_pressed;
                        true
                    }
                    KeyCode::Backquote => {
                        self.is_debug_pressed = is_pressed;
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
            camera.rotation.y += self.speed;
        }
        if self.is_left_pressed {
            camera.rotation.y -= self.speed;
        }
        if self.is_down_pressed {
            camera.rotation.x += self.speed;
        }     
        if self.is_up_pressed {
            camera.rotation.x -= self.speed;
        }
        if self.is_zcw_pressed {
            camera.rotation.z += self.speed;
        }     
        if self.is_zccw_pressed {
            camera.rotation.z -= self.speed;
        }

        // Recalculate the forward vector based on its new direction and magnitude
        let forward = Vector3::new(
            forward_mag * camera.rotation.x.cos() * camera.rotation.y.cos(),
            forward_mag * camera.rotation.x.sin(),
            forward_mag * camera.rotation.x.cos() * camera.rotation.y.sin(),
        );        
        // Reposition eye so that forward points at the target again
        camera.eye = camera.target - forward;
        camera.up = self.recalculate_up(forward, camera);
    }

    fn recalculate_up(&self, forward: Vector3<f32>, camera: &mut Camera) -> Vector3<f32> {
        // Recalculates up vector based on new rotations

        // Precompute values which are used a lot (and expensive)
        let camera_rotation_x = camera.rotation.x.rem_euclid(2.0 * PI);
        let sin_z = camera.rotation.z.sin();
        let cos_z = camera.rotation.z.cos();

        // Calculate the right vector
        // We use a global up vector because the real up vector actually doesn't effect the right vector (think about it)
        let mut right = forward.cross(Vector3::new(0.0, 1.0, 0.0)).normalize();
        // Change the signs of x and z so they work with every rotation
        // (each octant has different signs that follow this rule based on the forward vector)
        right.x = right.x.abs() * forward.z.signum();
        right.z = right.z.abs() * -forward.x.signum();

        // Calculate the up vector similarly to the right vector, only with different signs
        let mut up = forward.cross(right).normalize();
        up.x = up.x.abs() * forward.x.signum();
        up.y = up.y.abs();
        up.z = up.z.abs() * forward.z.signum();

        // Flip the up vector values if the camera is rotated upside down by the x axis
        // These fractions of PI come from trial and error and seeing which rotations break the up vector
        // If anyone knows their significance, please tell me (maybe I messed up the octant signs?)
        if (camera_rotation_x > 0.25 * PI && camera_rotation_x <= 0.5 * PI)
        || (camera_rotation_x >= 0.75 * PI && camera_rotation_x < 1.5 * PI) { up *= -1.0; }

        // Rotate the right vector around the forward vector
        // Effectively applies z rotation after the fact, 
        // so we dont have to deal with that messing up the previous calculations
        let forward_dot = forward.dot(forward);
        let parallel = (right.dot(forward) / forward_dot) * forward;
        let orthogonal = right - parallel;
        let w = forward.cross(orthogonal);
        let orthogonal_magnitude = orthogonal.magnitude();

        let x1 = cos_z / orthogonal_magnitude;
        let x2 = sin_z / w.magnitude();
        let orthogonal_rotated = orthogonal_magnitude * (x1 * orthogonal + x2 * w);
        right = orthogonal_rotated + parallel;

        // Rotate the up vector the same way
        let parallel = (up.dot(forward) / forward_dot) * forward;
        let orthogonal = up - parallel;
        let w = forward.cross(orthogonal);
        let orthogonal_magnitude = orthogonal.magnitude();

        let x1 = cos_z / orthogonal_magnitude;
        let x2 = sin_z / w.magnitude();
        let orthogonal_rotated = orthogonal_magnitude * (x1 * orthogonal + x2 * w);
        up = orthogonal_rotated + parallel;

        if self.is_debug_pressed {
            println!(
                "UP: {:#?} \nFORWARD: {:#?} \nRIGHT: {:#?} \nROT: {:#?} \nEYE: {:#?} \nTARGET: {:#?}",
                camera.up, forward.normalize(), right, camera.rotation, camera.eye, camera.target
            );
        }

        up
    }
}
