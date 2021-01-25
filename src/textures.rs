use crate::sdl2::render::Texture;

pub struct Tex{
    sprite: [Texture; 2],
}
impl Tex{
    pub fn new(texture_1: Texture, texture_2: Texture) -> Tex{
        Tex{
            sprite: [texture_1, texture_2],
        }
    }
}