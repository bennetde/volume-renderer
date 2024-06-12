#include "raylib.h"
#include "camera.h"
//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
int main(void)
{
    // Initialization
    //--------------------------------------------------------------------------------------
    const int screenWidth = 800;
    const int screenHeight = 450;

    InitWindow(screenWidth, screenHeight, "raylib [core] example - basic window");

    // Define the camera to look into our 3d world
    Camera3D camera = { 0 };
    camera.position = (Vector3){ 10.0f, 10.0f, 10.0f }; // Camera position
    camera.target = (Vector3){ 0.0f, 0.0f, 0.0f };      // Camera looking at point
    camera.up = (Vector3){ 0.0f, 1.0f, 0.0f };          // Camera up vector (rotation towards target)
    camera.fovy = 45.0f;                                // Camera field-of-view Y
    camera.projection = CAMERA_PERSPECTIVE;             // Camera projection type

    const char* fragmentShaderLoc = TextFormat("%sshaders/raymarching.fs", ASSETS_PATH);
    Shader shader = LoadShader(0, fragmentShaderLoc);

    Vector3 cubePosition = { 0.0f, 0.0f, 0.0f};

    DisableCursor();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while (!WindowShouldClose())    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        float delta = GetFrameTime();
        UpdateFreeCamera(&camera, delta);

        // Draw
        //----------------------------------------------------------------------------------
        BeginDrawing();

            // ClearBackground(RAYWHITE);

            // BeginMode3D(camera);

            //     DrawCube(cubePosition, 2.0f, 2.0f, 2.0f, RED);
            //     DrawCubeWires(cubePosition, 2.0f, 2.0f ,2.0f, MAROON);
            //     DrawGrid(10, 1.0f);
                
            // EndMode3D();

            BeginShaderMode(shader);
                DrawRectangleRec((Rectangle){0,0, (float)screenWidth, (float)screenHeight},WHITE);
                // DrawPlane()
            EndShaderMode();

        EndDrawing();
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    CloseWindow();        // Close window and OpenGL context
    //--------------------------------------------------------------------------------------

    return 0;
}