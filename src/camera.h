#pragma once
#include "raylib.h"

void UpdateFreeCamera(Camera* camera, float delta);
Matrix GetProjectionMatrix(Camera* camera, float aspectRatio);