use bevy::ecs::component::Component;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::tess::math::Angle;
use derive_more::From;

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct AngularVelocity(pub f32);

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct Damping(pub f32);

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct SpeedLimit(pub f32);

#[derive(Debug)]
pub enum DriveMode {
    Off,
    Propulsion,
    Reverse,
}

#[derive(Debug)]
pub enum ThrustersMode {
    Off,
    Left,
    Right,
}

#[derive(Debug, Component)]
pub struct Drive {
    pub mode: DriveMode,
    pub propulsion_force: f32,
    pub reverse_force: f32,
}
impl Drive {
    pub fn new(propulsion_force: f32, reverse_force: f32) -> Self {
        Drive {
            mode: DriveMode::Off,
            propulsion_force,
            reverse_force,
        }
    }
}

#[derive(Debug, Component)]
pub struct SideThrusters {
    pub mode: ThrustersMode,
    pub force: f32,
}
impl SideThrusters {
    pub fn new(force: f32) -> Self {
        SideThrusters {
            mode: ThrustersMode::Off,
            force,
        }
    }
}

#[derive(Debug, Component, Default, Deref, DerefMut, From)]
pub struct SteeringControl(Angle);

pub fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, Option<&AngularVelocity>, Option<&Velocity>)>,
) {
    for (mut transform, angular_velocity, velocity) in query.iter_mut() {
        if let Some(AngularVelocity(vel)) = angular_velocity {
            transform.rotate(Quat::from_rotation_z(vel * time.delta_seconds()))
        }
        if let Some(Velocity(vel)) = velocity {
            transform.translation.x += vel.x * time.delta_seconds();
            transform.translation.y += vel.y * time.delta_seconds();
        }
    }
}

pub fn damping_system(mut query: Query<(&mut Velocity, &Damping)>) {
    for (mut velocity, damping) in query.iter_mut() {
        velocity.0 *= damping.0;
    }
}

pub fn drive_control_system(mut query: Query<&mut Drive>, keyboard: Res<Input<KeyCode>>) {
    for mut drive in query.iter_mut() {
        drive.mode = if keyboard.pressed(KeyCode::Up) {
            DriveMode::Propulsion
        } else if keyboard.pressed(KeyCode::Down) {
            DriveMode::Reverse
        } else {
            DriveMode::Off
        }
    }
}

pub fn side_thruster_control_system(
    mut query: Query<&mut SideThrusters>,
    keyboard: Res<Input<KeyCode>>,
) {
    for mut thrusters in query.iter_mut() {
        thrusters.mode = if keyboard.pressed(KeyCode::Q) {
            ThrustersMode::Left
        } else if keyboard.pressed(KeyCode::E) {
            ThrustersMode::Right
        } else {
            ThrustersMode::Off
        }
    }
}

pub fn drive_system(mut query: Query<(&mut Velocity, &Transform, &Drive)>) {
    for (mut velocity, transform, drive) in query.iter_mut() {
        match drive.mode {
            DriveMode::Off => return,
            DriveMode::Propulsion => {
                // what the fuck is this quat shit
                // changed from Vec3::X to -Vec::Y and now this shit works wtf?
                let direction = transform.rotation * -Vec3::Y;
                velocity.x += direction.x * drive.propulsion_force;
                velocity.y += direction.y * drive.propulsion_force;
            }
            DriveMode::Reverse => {
                let direction = transform.rotation * -Vec3::Y;
                velocity.x += -(direction.x * drive.reverse_force);
                velocity.y += -(direction.y * drive.reverse_force);
            }
        }
    }
}

pub fn side_thruster_system(mut query: Query<(&mut Velocity, &Transform, &SideThrusters)>) {
    for (mut velocity, transform, thruster) in query.iter_mut() {
        match thruster.mode {
            ThrustersMode::Off => return,
            ThrustersMode::Left => {
                let angle = 90.0_f32.to_radians();
                let rot = transform.rotation;
                let direction = rot.mul_quat(Quat::from_rotation_z(angle)) * -Vec3::Y;
                velocity.x += direction.x * thruster.force;
                velocity.y += direction.y * thruster.force;
            }
            ThrustersMode::Right => {
                let angle = 90.0_f32.to_radians();
                let rot = transform.rotation;
                let direction = rot.mul_quat(Quat::from_rotation_z(angle)) * -Vec3::Y;
                velocity.x += -(direction.x * thruster.force);
                velocity.y += -(direction.y * thruster.force);
            }
        }
    }
}

pub fn steering_control_system(
    mut query: Query<(&mut AngularVelocity, &SteeringControl)>,
    keyboard: Res<Input<KeyCode>>,
) {
    for (mut angular_velocity, steering_control) in query.iter_mut() {
        if keyboard.pressed(KeyCode::Left) {
            *angular_velocity = AngularVelocity::from(steering_control.0.get());
        } else if keyboard.pressed(KeyCode::Right) {
            *angular_velocity = AngularVelocity::from(-steering_control.0.get());
        } else {
            *angular_velocity = AngularVelocity::from(0.0);
        }
    }
}
