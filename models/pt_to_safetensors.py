# cd models
# 
# curl -LO https://huggingface.co/openai/clip-vit-base-patch32/resolve/main/pytorch_model.bin?download=true# ---
# ---
# python -m venv venv
# source venv/bin/activate.fish
# 
# pip install torch safetensors packaging numpy
# 
# python pt_to_safetensors.py
# ---
# # deactivate
# # rm -r venv
# # rm pytorch_model.bin
# # cd ..

import torch
from safetensors.torch import save_file

print("Start.")

print("Model load...")
path_to_torch_model = "pytorch_model.bin"
state_dict = torch.load(path_to_torch_model, map_location="cpu", weights_only=True)

print("Tensor adjust to contiguous...")
state_dict = {k: v.contiguous() for k, v in state_dict.items()}

print("Save...")
save_file(state_dict, "model.safetensors")

print("Done.")
