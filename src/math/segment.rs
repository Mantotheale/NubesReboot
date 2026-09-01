use std::fmt;
use crate::math::approx_eq::ApproxEq;
use crate::math::point2f::Point2f;
use crate::math::positive_f32::PositiveF32;
use crate::math::vec2f::Vec2f;

#[derive(Debug)]
pub struct Segment2fPointsCoincideError {
    origin: Point2f,
    destination: Point2f
}

impl fmt::Display for Segment2fPointsCoincideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Segment's defining points must not coincide, they were {:?}, {:?}", self.origin, self.destination)
    }
}

impl std::error::Error for Segment2fPointsCoincideError {}

#[derive(Copy, Clone)]
pub struct Segment2f {
    origin: Point2f,
    destination: Point2f
}

impl Segment2f {
    pub fn new(origin: Point2f, destination: Point2f, epsilon: PositiveF32) -> Result<Self, Segment2fPointsCoincideError> {
        if origin.approx_eq(&destination, epsilon) {
            Err(Segment2fPointsCoincideError { origin, destination })
        } else {
            Ok(Self { origin, destination })
        }
    }

    pub fn origin(&self) -> Point2f {
        self.origin
    }

    pub fn destination(&self) -> Point2f {
        self.destination
    }

    pub fn direction(&self) -> Vec2f {
        self.destination - self.origin
    }
    
    pub fn len(&self) -> f32 {
        self.origin.distance(&self.destination)    
    }
    
    pub fn left_normal(&self) -> Vec2f {
        self.direction().left_normal()
    }

    pub fn right_normal(&self) -> Vec2f {
        self.direction().right_normal()
    }
}