use crate::fixed64::Fixed64;

use super::observation::Object;

pub struct VisionCone {
    pub max_distance: Fixed64,
    pub fov_degrees: u16,
}

impl VisionCone {
    pub fn can_see(&self, from: (Fixed64, Fixed64), to: (Fixed64, Fixed64)) -> bool {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let distance_sq = dx * dx + dy * dy;
        let max_dist_sq = self.max_distance * self.max_distance;

        distance_sq <= max_dist_sq
    }

    pub fn get_visible_objects(
        &self,
        from: (Fixed64, Fixed64),
        all_objects: &[Object],
    ) -> Vec<Object> {
        all_objects
            .iter()
            .filter(|obj| self.can_see(from, obj.position))
            .cloned()
            .collect()
    }
}
