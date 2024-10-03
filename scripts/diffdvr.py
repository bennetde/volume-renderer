import sys
import os
import time
import array

import torch.nn.functional
sys.path.append(os.getcwd())

from netCDF4 import Dataset

from pathlib import Path
import numpy as np
import torch
import matplotlib.pyplot as plt
import matplotlib.animation
import tqdm
import json
import pyrenderer

from PIL import Image

def make_real3(vector):
  return pyrenderer.real3(vector[0], vector[1], vector[2])


if __name__=='__main__':
    print(pyrenderer.__doc__)
    device = "cuda"
    dtype = torch.float32

    # Settings
    cameras_dir = r'C:\Users\Bennet\Documents\Projects\bachelorthesis\screenshots\tooth_cropped_1at_color\ground_truth\\'
    X = 92 # Volume resolution Width
    Y = 82 # Volume resolution Height
    Z = 159 # Volume resolution Depth
    H = 1024 # Screen Size Width
    W = 1024 # Screen Size Height
    DOWNSCALE = 1 # Downscale Output Volume, Set to 1 for no Downscaling
    EPOCHS = 32
    LEARING_RATE = 0.01
    # STEP_SIZE = 0.001
    STEP_SIZE = 0.25 / max(X, max(Y, Z))
    B = 1 # ?
    FILE_NAME = f'approx_{EPOCHS}epochs_{LEARING_RATE}lr_{STEP_SIZE}sz.nc' # Output file name

    # Select output type
    write_video = False
    write_hdf5 = False
    write_nc = True

    min_dim = min(Z, min(Y, X))
    div_scale = 1.3

    cameras_path = cameras_dir + 'cameras.json'
    with open(cameras_path) as f:
        cameras_json = json.load(f)

    look_at = cameras_json["look_at"]
    cameras_json = cameras_json["positions"]

    opacity_scaling = 1.0
    tf = torch.tensor([[
        [0.0,0.0,0.0,0.0 *opacity_scaling],
        [1.0,1.0,1.0,1.0 *opacity_scaling]
    ]], dtype=dtype, device=device)

    print("Create data set")
    volume_tensor = torch.ones((4, X // DOWNSCALE, Y // DOWNSCALE, Z // DOWNSCALE), dtype=dtype, device=device) * 0.5
    volume_tensor[3,:,:,:] = opacity_scaling

    print("Create renderer inputs")
    inputs = pyrenderer.RendererInputs()
    inputs.screen_size = pyrenderer.int2(W, H)
    inputs.volume = volume_tensor
    inputs.volume_filter_mode = pyrenderer.VolumeFilterMode.Preshaded
    inputs.box_min = pyrenderer.real3(-X / min_dim / 2 / div_scale, -Y / min_dim / 2 / div_scale, -Z / min_dim / 2 / div_scale)
    inputs.box_size = pyrenderer.real3(X / min_dim / div_scale, Y / min_dim / div_scale, Z / min_dim / div_scale)
    # inputs.step_size = 0.25 / max(X, max(Y, Z))
    inputs.step_size = STEP_SIZE
    inputs.tf_mode = pyrenderer.TFMode.Preshaded
    inputs.tf = tf
    inputs.blend_mode = pyrenderer.BlendMode.BeerLambert

    output_color_test = torch.empty(1, H, W, 4, dtype=dtype, device=device)
    output_termination_index_test = torch.empty(1, H, W, dtype=torch.int32, device=device)
    outputs_test = pyrenderer.RendererOutputs(output_color_test, output_termination_index_test)

    print("Create renderer outputs")
    output_color = torch.empty(1, H, W, 4, dtype=dtype, device=device)
    output_termination_index = torch.empty(1, H, W, dtype=torch.int32, device=device)
    outputs = pyrenderer.RendererOutputs(output_color, output_termination_index)

    adjoint_outputs = pyrenderer.AdjointOutputs()
    grad_volume = torch.zeros_like(volume_tensor)
    adjoint_outputs.has_volume_derivatives = True
    adjoint_outputs.adj_volume = grad_volume

    camera_ref_orientation = pyrenderer.Orientation.Ym
    camera_ref_center = torch.tensor([[0.0, 0.0, 0.0]], dtype=dtype, device=device)
    camera_ref_distance = torch.tensor([[0.7]], dtype=dtype, device=device)
    camera_test_pitch = torch.tensor([[np.radians(-45)]], dtype=dtype, device=device)
    camera_test_yaw = torch.tensor([[np.radians(-40)]], dtype=dtype, device=device)
    viewport_test = pyrenderer.Camera.viewport_from_sphere(
    camera_ref_center, camera_test_yaw, camera_test_pitch, camera_ref_distance, camera_ref_orientation)
    ray_start_test, ray_dir_test = pyrenderer.Camera.generate_rays(viewport_test, np.radians(45.0), W, H)
    camera_test_second_view = pyrenderer.CameraPerPixelRays(ray_start_test, ray_dir_test)

    
    class RendererDerivAdjoint(torch.autograd.Function):
        @staticmethod
        def forward(ctx, volume_tensor):
            inputs.volume = volume_tensor
            # render
            grad_volume.zero_()
            pyrenderer.Renderer.render_forward(inputs, outputs)
            return output_color
        @staticmethod
        def backward(ctx, grad_output_color):
            pyrenderer.Renderer.render_adjoint(inputs, outputs, grad_output_color, adjoint_outputs)
            return grad_volume
        
    rendererDeriv = RendererDerivAdjoint.apply

    class OptimModelVolume(torch.nn.Module):
        def __init__(self):
            super().__init__()
            self.sigmoid = torch.nn.Sigmoid()
        
        def forward(self, iteration, volume_tensor):
            color = rendererDeriv(volume_tensor)
            loss = torch.nn.functional.mse_loss(color, reference_color_gpu)
            return loss, volume_tensor, color
        
    model = OptimModelVolume()

    start_time = time.time()

    iterations = len(cameras_json) * EPOCHS
    # iterations = ca
    reconstructed_color = []
    reconstructed_loss = []
    second_view_images = []
    reference_color_images = []
    volume_tensor.requires_grad_()
    variables = []
    variables.append(volume_tensor)
    learning_rate = LEARING_RATE
    optimizer = torch.optim.Adam(variables, lr=learning_rate)
    print("%d iterations", range(iterations))
    for iteration in range(iterations):
        optimizer.zero_grad()

        img_idx = iteration % len(cameras_json)
        camera_json = cameras_json[img_idx]
        fg_image_path = cameras_dir + camera_json["img"]
        
        ref_img = Image.open(fg_image_path)
        reference_color_image = np.array(ref_img).astype(np.float32) / 255.0
        reference_color_image = reference_color_image.reshape((1, reference_color_image.shape[0], reference_color_image.shape[1], 4))
        reference_color_image = reference_color_image.transpose(0, 1, 2, 3)
        # imgplot = plt.imshow(reference_color_image)
        # plt.show()

        if write_video:
            reference_color_images.append(reference_color_image[0,:,:,0:3])
        reference_color_gpu = torch.tensor(reference_color_image, device=device)

        fovy = camera_json["fovy"]
        camera_origin = np.array(camera_json["position"])
        camera_origin[1] = -camera_origin[1]
        camera_origin[0] = -camera_origin[0]
        # camera_right = np.array(camera_json["right"])
        camera_up = np.array(camera_json["up"])
        # camera_up[0] = -camera_up[0]
        camera_up[0] = 0
        camera_up[1] = 1
        camera_up[2] = 0
        # camera_up[1] = -camera_up[1]
        # camera_up[1] = -camera_up[1]
        # camera_front = np.array(camera_json["front"])
        invViewMatrix = pyrenderer.Camera.compute_matrix(
            make_real3(camera_origin), make_real3(look_at), make_real3(camera_up), fovy, W, H, 0.1, 1000.0
        )
        inputs.camera = invViewMatrix
        inputs.camera_mode = pyrenderer.CameraMode.InverseViewMatrix

        loss, volume_tensor, color = model(iteration, volume_tensor)
        if write_video:
            reconstructed_color.append(color.detach().cpu().numpy()[0,:,:,0:3])
        reconstructed_loss.append(loss.item())
        loss.backward()
        optimizer.step()
        with torch.no_grad():
            inputs_cpy = inputs.clone()
            inputs_cpy.camera_mode = pyrenderer.CameraMode.RayStartDir
            inputs_cpy.camera = camera_test_second_view
            pyrenderer.Renderer.render_forward(inputs_cpy, outputs_test)
            test_image = output_color_test.cpu().numpy()[0]
            if write_video:
                second_view_images.append(test_image)
        print("Iteration % 4d, %s, Loss %7.5f"%(iteration, camera_json["img"], loss.item()))
    
    elapsed_time = time.time() - start_time
    print(f'Elapsed time optimization: {elapsed_time}s')

    if write_nc:
        ncfile = Dataset(FILE_NAME, mode='w', format='NETCDF4_CLASSIC')
        cdim = ncfile.createDimension('c', 4)
        zdim = ncfile.createDimension('z', Z // DOWNSCALE)
        ydim = ncfile.createDimension('y', Y // DOWNSCALE)
        xdim = ncfile.createDimension('x', X // DOWNSCALE)
        outfield_color = ncfile.createVariable('color', np.float32, ('c', 'z', 'y', 'x'))
        #outfield_color[:, :, :, :] = volume_tensor.detach().cpu().numpy().flatten('F')
        #outfield_color.flatten('F') = volume_tensor.detach().cpu().numpy().flatten('F')
        outfield_color[:, :, :, :] = np.flip(np.flip(volume_tensor.detach().cpu().numpy().transpose(0, 3, 2, 1),2),3)
        ncfile.close()

    if write_video:
        print("Visualize Optimization")
        fig, axs = plt.subplots(3, 1, figsize=(8,6))
        axs[0].imshow(reference_color_images[0][:,:,0:3])
        axs[1].imshow(reconstructed_color[0])
        axs[2].imshow(second_view_images[0][:,:,0:3])
        axs[0].set_title("Color")
        axs[0].set_ylabel("Reference")
        axs[1].set_ylabel("Optimization")
        axs[2].set_ylabel("Side View")
        for i in range(3):
            axs[i].set_xticks([])
            #if j==0: axs[i,j].set_yticks([])
            axs[i].set_yticks([])
        fig.suptitle("Iteration % 4d, Loss: %7.3f" % (0, reconstructed_loss[0]))
        fig.tight_layout()

        print("Write frames")
        with tqdm.tqdm(total=len(reconstructed_color)) as pbar:
            def update(frame):
                axs[0].clear()
                axs[0].imshow(reference_color_images[frame][:,:,0:3])
                axs[1].clear()
                axs[1].imshow(reconstructed_color[frame])
                axs[2].clear()
                axs[2].imshow(second_view_images[frame][:,:,0:3])
                fig.suptitle("Iteration % 4d, Loss: %7.5f"%(frame, reconstructed_loss[frame]))
                if frame > 0:
                    pbar.update(1)
            anim = matplotlib.animation.FuncAnimation(
                fig, update, frames=len(reconstructed_color), cache_frame_data=False)
            anim.save(f"test_preshaded.mp4")
    
    pyrenderer.cleanup()
