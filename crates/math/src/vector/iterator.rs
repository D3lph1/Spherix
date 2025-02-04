use crate::vector::VectorPlain;

pub trait SquareIter<V: VectorPlain>: Iterator<Item = V> {
    fn center(&self) -> V;

    fn radius(&self) -> usize;
}

#[derive(Clone)]
pub struct UnorderedSquareIter<V> {
    center: V,
    radius: i32,
    i: i32,
}

impl <V: VectorPlain> UnorderedSquareIter<V> {
    pub fn new(center: V, radius: usize) -> Self {
        Self {
            center,
            radius: radius as i32,
            i: 0,
        }
    }
}

impl <V: VectorPlain> Iterator for UnorderedSquareIter<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        let diameter = self.radius * 2 + 1;
        if self.i >= diameter * diameter {
            return None;
        }

        let x = self.i % diameter;
        let y = self.i / diameter;

        self.i += 1;

        let start_x = self.center.x() - self.radius;
        let start_y = self.center.z() - self.radius;

        Some(Self::Item::new_from(&self.center, start_x + x, start_y + y))
    }
}

impl <V: VectorPlain> SquareIter<V> for UnorderedSquareIter<V> {
    #[inline]
    fn center(&self) -> V {
        self.center.clone()
    }

    #[inline]
    fn radius(&self) -> usize {
        self.radius as usize
    }
}

#[derive(Clone)]
pub struct OrderedSquareIter<V> {
    center: V,
    radius: i32,
    i: i32,
    j: i32,
    r: i32,
    state: OrderedSquareIterState,
}

#[derive(Clone, PartialEq)]
enum OrderedSquareIterState {
    Start,
    XTop,
    YRight,
    XBottom,
    YLeft,
    EndIter,
}

impl <V: VectorPlain> OrderedSquareIter<V> {
    pub fn new(center: V, radius: usize) -> Self {
        let x = center.x();
        let y = center.z();

        Self {
            center,
            radius: radius as i32 + 1,
            i: x - 1,
            j: y - 1,
            r: 1,
            state: OrderedSquareIterState::Start,
        }
    }
}

impl <V: VectorPlain> Iterator for OrderedSquareIter<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == OrderedSquareIterState::Start {
            self.state = OrderedSquareIterState::XTop;
            return Some(V::new_from(&self.center, self.center.x(), self.center.z()));
        }

        if self.radius == 1 {
            return None
        }

        if self.state == OrderedSquareIterState::XTop {
            while self.i < self.center.x() + self.r {
                self.i += 1;

                return Some(V::new_from(&self.center, self.i, self.j));
            }

            self.state = OrderedSquareIterState::YRight;
        }

        if self.state == OrderedSquareIterState::YRight {
            while self.j < self.center.z() + self.r {
                self.j += 1;

                return Some(V::new_from(&self.center, self.i, self.j));
            }

            self.state = OrderedSquareIterState::XBottom;
        }

        if self.state == OrderedSquareIterState::XBottom {
            while self.i > self.center.x() - self.r {
                self.i -= 1;

                return Some(V::new_from(&self.center, self.i, self.j));
            }

            self.state = OrderedSquareIterState::YLeft;
        }

        if self.state == OrderedSquareIterState::YLeft {
            while self.j > self.center.z() - self.r {
                self.j -= 1;

                return Some(V::new_from(&self.center, self.i, self.j));
            }

            self.state = OrderedSquareIterState::EndIter;
        }

        self.r += 1;

        if self.r == self.radius {
            return None;
        }

        self.j -= 1;
        self.state = OrderedSquareIterState::XTop;
        return Some(V::new_from(&self.center, self.i, self.j));
    }
}

impl <V: VectorPlain> SquareIter<V> for OrderedSquareIter<V> {
    #[inline]
    fn center(&self) -> V {
        self.center.clone()
    }

    #[inline]
    fn radius(&self) -> usize {
        self.radius as usize - 1
    }
}

/// The structure contains `center` field that duplicates value from
/// the corresponding field of the inner iterator. The only purpose
/// of it - increase performance by eliminating unnecessary cloning.
#[derive(Clone)]
pub struct RadialIter<V: VectorPlain, T: SquareIter<V>> {
    inner: T,
    center: V
}

impl <V: VectorPlain, T: SquareIter<V>> RadialIter<V, T> {
    pub fn new(square_iter: T) -> Self {
        Self {
            center: square_iter.center().clone(),
            inner: square_iter,
        }
    }

    /// Fast implementation of Euclidean distance calculation that uses only integer
    /// arithmetics. The function does not contain any squared root call because
    /// such expensive operation is not necessary as we have to just compare two
    /// values, not to obtain exact distance value.
    fn within_radius(center: &V, point: &V, diameter: i32) -> bool {
        let dx = center.x() - point.x();
        let dy = center.z() - point.z();
        let distance_squared = dx * dx + dy * dy;

        return 4 * distance_squared <= diameter * diameter;
    }
}

impl <V: VectorPlain, T: SquareIter<V>> Iterator for RadialIter<V, T> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        let diameter = 2 * self.inner.radius() as i32 + 1;

        loop {
            let v = self.inner.next()?;

            if Self::within_radius(&v, &self.center, diameter) {
                return Some(v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vector::{OrderedSquareIter, RadialIter, UnorderedSquareIter, Vector2};
    use spherix_util::iters_equal_anyorder;

    #[test]
    fn unordered_square_iter() {
        let center = Vector2::new(0, 0);
        let iter = UnorderedSquareIter::new(center.clone(), 2);
        let vec: Vec<Vector2> = iter.collect();
        assert_eq!(25, vec.len());

        let iter = UnorderedSquareIter::new(center, 3);
        let vec: Vec<Vector2> = iter.collect();
        assert!(iters_equal_anyorder(
            vec![
                Vector2::new(0, 0),
                Vector2::new(-1, -1),
                Vector2::new(0, -1),
                Vector2::new(-1, 0),
                Vector2::new(-1, 1),
                Vector2::new(0, 1),
                Vector2::new(1, 0),
                Vector2::new(1, -1),
                Vector2::new(1, 1),
                Vector2::new(-2, -2),
                Vector2::new(-2, -1),
                Vector2::new(-2, 0),
                Vector2::new(-2, 1),
                Vector2::new(-2, 2),
                Vector2::new(-1, 2),
                Vector2::new(0, 2),
                Vector2::new(1, 2),
                Vector2::new(2, 2),
                Vector2::new(2, 1),
                Vector2::new(2, 0),
                Vector2::new(2, -1),
                Vector2::new(2, -2),
                Vector2::new(-1, -2),
                Vector2::new(0, -2),
                Vector2::new(1, -2),
                Vector2::new(-3, -3),
                Vector2::new(-3, -2),
                Vector2::new(-3, -1),
                Vector2::new(-3, 0),
                Vector2::new(-3, 1),
                Vector2::new(-3, 2),
                Vector2::new(-3, 3),
                Vector2::new(-2, 3),
                Vector2::new(-1, 3),
                Vector2::new(0, 3),
                Vector2::new(1, 3),
                Vector2::new(2, 3),
                Vector2::new(3, 3),
                Vector2::new(3, 2),
                Vector2::new(3, 1),
                Vector2::new(3, 0),
                Vector2::new(3, -1),
                Vector2::new(3, -2),
                Vector2::new(3, -3),
                Vector2::new(-2, -3),
                Vector2::new(-1, -3),
                Vector2::new(0, -3),
                Vector2::new(1, -3),
                Vector2::new(2, -3),
            ].into_iter(),
            vec.into_iter(),
        ));
    }

    #[test]
    fn ordered_square_iter_origin() {
        let center = Vector2::new(0, 0);
        let iter = OrderedSquareIter::new(center.clone(), 2);
        let vec: Vec<Vector2> = iter.collect();
        assert_eq!(25, vec.len());

        let iter = OrderedSquareIter::new(center, 3);
        let vec: Vec<Vector2> = iter.collect();
        assert_eq!(Vector2::new(0, 0), vec[0]);
        assert!(iters_equal_anyorder(
            vec![
                Vector2::new(-1, -1),
                Vector2::new(0, -1),
                Vector2::new(-1, 0),
                Vector2::new(-1, 1),
                Vector2::new(0, 1),
                Vector2::new(1, 0),
                Vector2::new(1, -1),
                Vector2::new(1, 1),
            ].into_iter(),
            vec[1..=8].to_vec().into_iter(),
        ));

        assert!(iters_equal_anyorder(
            vec![
                Vector2::new(-2, -2),
                Vector2::new(-2, -1),
                Vector2::new(-2, 0),
                Vector2::new(-2, 1),
                Vector2::new(-2, 2),
                Vector2::new(-1, 2),
                Vector2::new(0, 2),
                Vector2::new(1, 2),
                Vector2::new(2, 2),
                Vector2::new(2, 1),
                Vector2::new(2, 0),
                Vector2::new(2, -1),
                Vector2::new(2, -2),
                Vector2::new(-1, -2),
                Vector2::new(0, -2),
                Vector2::new(1, -2),
            ].into_iter(),
            vec[9..=24].to_vec().into_iter(),
        ));

        assert!(iters_equal_anyorder(
            vec![
                Vector2::new(-3, -3),
                Vector2::new(-3, -2),
                Vector2::new(-3, -1),
                Vector2::new(-3, 0),
                Vector2::new(-3, 1),
                Vector2::new(-3, 2),
                Vector2::new(-3, 3),
                Vector2::new(-2, 3),
                Vector2::new(-1, 3),
                Vector2::new(0, 3),
                Vector2::new(1, 3),
                Vector2::new(2, 3),
                Vector2::new(3, 3),
                Vector2::new(3, 2),
                Vector2::new(3, 1),
                Vector2::new(3, 0),
                Vector2::new(3, -1),
                Vector2::new(3, -2),
                Vector2::new(3, -3),
                Vector2::new(-2, -3),
                Vector2::new(-1, -3),
                Vector2::new(0, -3),
                Vector2::new(1, -3),
                Vector2::new(2, -3),
            ].into_iter(),
            vec[25..=48].to_vec().into_iter(),
        ));
    }

    #[test]
    fn ordered_square_iter_neg() {
        let center = Vector2::new(22, -3);
        let iter = OrderedSquareIter::new(center.clone(), 3);
        let vec: Vec<Vector2> = iter.collect();
        assert_eq!(49, vec.len());

        assert_eq!(Vector2::new(22, -3), vec[0]);
        assert!(iters_equal_anyorder(
            vec![
                Vector2::new(21, -4),
                Vector2::new(21, -3),
                Vector2::new(21, -2),
                Vector2::new(22, -4),
                Vector2::new(22, -2),
                Vector2::new(23, -4),
                Vector2::new(23, -3),
                Vector2::new(23, -2),
            ].into_iter(),
            vec[1..=8].to_vec().into_iter(),
        ));

        assert!(iters_equal_anyorder(
            vec![
                Vector2::new(20, -5),
                Vector2::new(20, -4),
                Vector2::new(20, -3),
                Vector2::new(20, -2),
                Vector2::new(20, -1),
                Vector2::new(21, -5),
                Vector2::new(21, -1),
                Vector2::new(22, -5),
                Vector2::new(22, -1),
                Vector2::new(23, -5),
                Vector2::new(23, -1),
                Vector2::new(24, -5),
                Vector2::new(24, -4),
                Vector2::new(24, -3),
                Vector2::new(24, -2),
                Vector2::new(24, -1),
            ].into_iter(),
            vec[9..=24].to_vec().into_iter(),
        ));

        assert!(iters_equal_anyorder(
            vec![
                Vector2::new(19, -6),
                Vector2::new(19, -5),
                Vector2::new(19, -4),
                Vector2::new(19, -3),
                Vector2::new(19, -2),
                Vector2::new(19, -1),
                Vector2::new(19, 0),
                Vector2::new(20, -6),
                Vector2::new(20, 0),
                Vector2::new(21, -6),
                Vector2::new(21, 0),
                Vector2::new(22, -6),
                Vector2::new(22, 0),
                Vector2::new(23, -6),
                Vector2::new(23, 0),
                Vector2::new(24, -6),
                Vector2::new(24, 0),
                Vector2::new(25, -6),
                Vector2::new(25, -5),
                Vector2::new(25, -4),
                Vector2::new(25, -3),
                Vector2::new(25, -2),
                Vector2::new(25, -1),
                Vector2::new(25, 0),
            ].into_iter(),
            vec[25..=48].to_vec().into_iter(),
        ));
    }

    #[test]
    fn radial_iter() {
        let center = Vector2::new(0, 0);
        let iter = RadialIter::new(OrderedSquareIter::new(center, 2));
        let vec: Vec<Vector2> = iter.collect();

        assert_eq!(21, vec.len());

        assert!(iters_equal_anyorder(
            vec![
                Vector2::new(0, 0),
                Vector2::new(0, -1),
                Vector2::new(0, 1),
                Vector2::new(-1, 0),
                Vector2::new(1, 0),
                Vector2::new(1, 1),
                Vector2::new(1, -1),
                Vector2::new(-1, 1),
                Vector2::new(-1, -1),
            ].into_iter(),
            vec[0..=8].to_vec().into_iter(),
        ));
    }
}
