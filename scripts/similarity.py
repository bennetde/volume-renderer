#!/usr/bin/python3

# BSD 2-Clause License
# 
# Copyright (c) 2021, Christoph Neuhauser
# All rights reserved.
# 
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
# 
# 1. Redistributions of source code must retain the above copyright notice, this
#    list of conditions and the following disclaimer.
# 
# 2. Redistributions in binary form must reproduce the above copyright notice,
#    this list of conditions and the following disclaimer in the documentation
#    and/or other materials provided with the distribution.
# 
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
# AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
# IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

import sys
import math
import skimage
import skimage.io
import skimage.metrics
import itertools


if __name__ == '__main__':
    dir_gt = sys.argv[1]
    dir_approx = sys.argv[2]

    if not dir_gt.endswith('/'):
        dir_gt += '/'
    
    if not dir_approx.endswith('/'):
        dir_approx += '/'

    total_mse = 0.0
    total_rmse = 0.0
    total_psnr = 0.0
    total_ssim = 0.0
    TOTAL_IMAGES = 8 * 7 

    print("filename, mse, rmse, psnr, ssim")
    for combo in itertools.product(range(0,8),range(1,8)):
        filename = f'{combo[0]}-{combo[1]}.png'
        filename_gt = dir_gt + filename
        filename_approx = dir_approx + filename

        img_gt = skimage.io.imread(filename_gt)
        img_approx = skimage.io.imread(filename_approx)
        mse = skimage.metrics.mean_squared_error(img_gt, img_approx)
        rmse = math.sqrt(mse)
        psnr = skimage.metrics.peak_signal_noise_ratio(img_gt, img_approx)
        data_range=img_gt.max() - img_approx.min()
        ssim = skimage.metrics.structural_similarity(img_gt, img_approx, data_range=data_range, channel_axis=-1, multichannel=True)

        total_mse += mse
        total_rmse += rmse
        total_psnr += psnr
        total_ssim += ssim

        print(f'{filename}, {mse}, {rmse}, {psnr}, {ssim}')


    average_mse = total_mse / TOTAL_IMAGES
    average_rmse = total_rmse / TOTAL_IMAGES
    average_psnr = total_psnr / TOTAL_IMAGES
    average_ssim = total_ssim / TOTAL_IMAGES

    print(f'average, {average_mse}, {average_rmse}, {average_psnr}, {average_ssim}')