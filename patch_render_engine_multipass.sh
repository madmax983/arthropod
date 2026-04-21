sed -i 's/let mut instances_buffer = Vec::with_capacity(64);/let mut tmp_instances_buffer = Vec::with_capacity(64);/g' crates/render-engine/src/backend/wgpu/multipass_executor.rs
sed -i 's/let mut path_batches_buffer = Vec::with_capacity(16);/let mut tmp_path_batches_buffer = Vec::with_capacity(16);/g' crates/render-engine/src/backend/wgpu/multipass_executor.rs
sed -i 's/instances_buffer\.clear();/tmp_instances_buffer.clear();/g' crates/render-engine/src/backend/wgpu/multipass_executor.rs
sed -i 's/path_batches_buffer\.clear();/tmp_path_batches_buffer.clear();/g' crates/render-engine/src/backend/wgpu/multipass_executor.rs
sed -i 's/&mut instances_buffer,/\&mut tmp_instances_buffer,/g' crates/render-engine/src/backend/wgpu/multipass_executor.rs
sed -i 's/&mut path_batches_buffer,/\&mut tmp_path_batches_buffer,/g' crates/render-engine/src/backend/wgpu/multipass_executor.rs
