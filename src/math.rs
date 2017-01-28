pub use num_traits::{Zero, One};
use na::{Column, Matrix2, Matrix3, Inverse, BaseFloat, Vector2, Vector3, ToHomogeneous};

pub type Affine2<N> = Matrix3<N>;

pub trait Affine<N> {
    fn new(l: Matrix2<N>, t: Vector2<N>) -> Self;
    fn id() -> Self;
    fn inv(self) -> Self;
    fn translate(x: Vector2<N>) -> Self;
    fn rotate(x: N) -> Self;
    fn scale(x: Vector2<N>) -> Self;
}

impl<N: BaseFloat> Affine<N> for Affine2<N> {
    fn new(l: Matrix2<N>, t: Vector2<N>) -> Self {
        let mut s = l.to_homogeneous();
        s.set_column(2, Vector3::new(t.x, t.y, N::one()));
        s
    }
    fn id() -> Self {
        Matrix3::one()
    }
    fn inv(self) -> Self {
        self.inverse().expect("non-invertible matrix")
    }
    fn translate(x: Vector2<N>) -> Self {
        Affine::new(Matrix2::one(), x)
    }
    fn scale(s: Vector2<N>) -> Self {
        let _0 = N::zero();
        Affine::new(Matrix2::new(s.x, _0, _0, s.y), Zero::zero())
    }
    fn rotate(x: N) -> Self {
        use na::{Rotation2, Vector1};
        Affine::new(*Rotation2::new(Vector1::new(x)).submatrix(), Zero::zero())
    }
}
