// Library
use vek::*;

// Project
use common::{
    terrain::Block,
    vol::{BaseVol, ReadVol, SizedVol, VolSize, Vox},
    volumes::{dyna::Dyna, vol_map_2d::VolMap2d, vol_map_3d::VolMap3d},
};

// Crate
use crate::{
    mesh::{vol, Meshable},
    render::{self, Mesh, Quad, TerrainPipeline},
};

type TerrainVertex = <TerrainPipeline as render::Pipeline>::Vertex;

impl<M> Meshable for Dyna<Block, M> {
    type Pipeline = TerrainPipeline;
    type Supplement = ();

    fn generate_mesh(&self, _: Self::Supplement) -> Mesh<Self::Pipeline> {
        let mut mesh = Mesh::new();

        for pos in self
            .iter_positions()
            .filter(|pos| pos.map(|e| e >= 1).reduce_and())
            .filter(|pos| {
                pos.map2(self.get_size(), |e, sz| e < sz as i32 - 1)
                    .reduce_and()
            })
        {
            let offs = pos.map(|e| e as f32 - 1.0);

            if let Some(col) = self.get(pos).ok().and_then(|vox| vox.get_color()) {
                let col = col.map(|e| e as f32 / 255.0);

                vol::push_vox_verts(&mut mesh, self, pos, offs, col, TerrainVertex::new, true);
            }
        }

        mesh
    }
}

impl<V: BaseVol<Vox = Block> + ReadVol, S: VolSize + Clone> Meshable for VolMap2d<V, S> {
    type Pipeline = TerrainPipeline;
    type Supplement = Aabb<i32>;

    fn generate_mesh(&self, range: Self::Supplement) -> Mesh<Self::Pipeline> {
        let mut mesh = Mesh::new();

        for x in range.min.x + 1..range.max.x - 1 {
            for y in range.min.y + 1..range.max.y - 1 {
                for z in range.min.z..range.max.z {
                    let pos = Vec3::new(x, y, z);
                    let offs = (pos - range.min * Vec3::new(1, 1, 0)).map(|e| e as f32)
                        - Vec3::new(1.0, 1.0, 0.0);

                    if let Some(col) = self.get(pos).ok().and_then(|vox| vox.get_color()) {
                        let col = col.map(|e| e as f32 / 255.0);

                        vol::push_vox_verts(
                            &mut mesh,
                            self,
                            pos,
                            offs,
                            col,
                            TerrainVertex::new,
                            false,
                        );
                    }
                }
            }
        }
        mesh
    }
}

impl<V: BaseVol<Vox = Block> + ReadVol, S: VolSize + Clone> Meshable for VolMap3d<V, S> {
    type Pipeline = TerrainPipeline;
    type Supplement = Aabb<i32>;

    fn generate_mesh(&self, range: Self::Supplement) -> Mesh<Self::Pipeline> {
        let mut mesh = Mesh::new();

        let mut last_chunk_pos = self.pos_key(range.min);
        let mut last_chunk = self.get_key(last_chunk_pos);

        let size = range.max - range.min;
        for x in 1..size.x - 1 {
            for y in 1..size.y - 1 {
                for z in 1..size.z - 1 {
                    let pos = Vec3::new(x, y, z);

                    let new_chunk_pos = self.pos_key(range.min + pos);
                    if last_chunk_pos != new_chunk_pos {
                        last_chunk = self.get_key(new_chunk_pos);
                        last_chunk_pos = new_chunk_pos;
                    }
                    let offs = pos.map(|e| e as f32 - 1.0);
                    if let Some(chunk) = last_chunk {
                        let chunk_pos = Self::chunk_offs(range.min + pos);
                        if let Some(col) = chunk.get(chunk_pos).ok().and_then(|vox| vox.get_color())
                        {
                            let col = col.map(|e| e as f32 / 255.0);

                            vol::push_vox_verts(
                                &mut mesh,
                                self,
                                range.min + pos,
                                offs,
                                col,
                                TerrainVertex::new,
                                false,
                            );
                        }
                    } else {
                        if let Some(col) = self
                            .get(range.min + pos)
                            .ok()
                            .and_then(|vox| vox.get_color())
                        {
                            let col = col.map(|e| e as f32 / 255.0);

                            vol::push_vox_verts(
                                &mut mesh,
                                self,
                                range.min + pos,
                                offs,
                                col,
                                TerrainVertex::new,
                                false,
                            );
                        }
                    }
                }
            }
        }
        mesh
    }
}
