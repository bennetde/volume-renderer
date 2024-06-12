#include "camera.h"
#include "rcamera.h"
#include "raymath.h"

#define CAMERA_MOVE_SPEED 10.0f
#define CAMERA_MOVE_SPEED_MULTIPLIER 2.0f
#define CAMERA_MOUSE_MOVE_SENSITIVITY 0.1f

void UpdateFreeCamera(Camera* camera, float delta) {
        bool moveInWorldPlane = true;

        float moveSpeed = CAMERA_MOVE_SPEED * delta;
        float rotationSpeed = CAMERA_MOUSE_MOVE_SENSITIVITY;

        Vector3 movement = {0.0f, 0.f, 0.f};
        Vector3 rotation = {0.f, 0.f, 0.f};

        // Keyboard support
        if (IsKeyDown(KEY_W)) movement.x += 1.0f;
        if (IsKeyDown(KEY_A)) movement.y += -1.0;
        if (IsKeyDown(KEY_S)) movement.x += -1.0f;
        if (IsKeyDown(KEY_D)) movement.y += 1.0f;

        if (IsKeyDown(KEY_LEFT_SHIFT)) moveSpeed *= CAMERA_MOVE_SPEED_MULTIPLIER;

        movement = Vector3Scale(Vector3Normalize(movement), moveSpeed);

        if (IsKeyDown(KEY_SPACE)) movement.z += moveSpeed;
        if (IsKeyDown(KEY_LEFT_CONTROL)) movement.z += -moveSpeed;

        // Mouse support
        Vector2 mousePositionDelta = GetMouseDelta(); 
        rotation.x = mousePositionDelta.x * rotationSpeed;
        rotation.y = mousePositionDelta.y * rotationSpeed;

        float zoom = -GetMouseWheelMove();

        UpdateCameraPro(camera, movement, rotation, zoom);
}