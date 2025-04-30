#include "MainWindow.h"

#include <filesystem>
#include <iostream>

MainWindow::MainWindow()
{
// Don't set a custom font, use the default one
// ImGui::SetCurrentFont(LoadFonts(22.0f));

// Initialize the font system
LoadFonts(22.0f);

Controller = std::make_unique<NewDasherController>();
Controller->Initialize();
}

// Input handling is now done in the NewDasherController

bool MainWindow::render(float DeltaTime)
{
    static ImGuiWindowFlags flags = ImGuiWindowFlags_NoDecoration | ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoSavedSettings | ImGuiWindowFlags_NoBackground;

    const ImGuiViewport* viewport = ImGui::GetMainViewport();
    const ImVec2 spacing = ImGui::GetStyle().ItemSpacing;
    ImGui::SetNextWindowPos(viewport->WorkPos);
    ImGui::SetNextWindowSize(viewport->WorkSize);

    if (ImGui::Begin("MainWindow", nullptr, flags))
    {
        if(ImGui::BeginMainMenuBar())
        {
            if (ImGui::BeginMenu("File"))
            {
                if (ImGui::MenuItem("Quit", "CTRL+Q"))
                {
                    return false;
                }
                ImGui::EndMenu();
            }
            if (ImGui::BeginMenu("Edit"))
            {
                if (ImGui::MenuItem("Copy", "CTRL+C"))
                {
                    CopyTextToClipboard(*Controller->GetBufferRef());
                }
                if (ImGui::MenuItem("Clear", "CTRL+R"))
                {
                    ClearBuffer();
                }
                ImGui::EndMenu();
            }
            if (ImGui::BeginMenu("Help"))
            {
                if (ImGui::MenuItem("Demo Window"))
                {
                    DisplayDemoWindow = !DisplayDemoWindow;
                }
                ImGui::EndMenu();
            }

            ImGui::EndMainMenuBar();
        }

        ImVec2 MainWindowSize = ImGui::GetContentRegionMax();
        ImGui::InputTextMultiline("##DasherOutput", Controller->GetBufferRef(), ImVec2(-(MainWindowSize.x*0.1f + spacing.x), MainWindowSize.y * 0.1f));
        ImGui::SameLine();
        ImGui::BeginGroup();
            float CursorPosXGroup = ImGui::GetCursorScreenPos().x;
            ImVec2 ButtonSize = ImVec2(-FLT_MIN, MainWindowSize.y*0.05f - spacing.y * 0.5f);
            if(ImGui::Button("Copy", ButtonSize))
            {
                CopyTextToClipboard(*Controller->GetBufferRef());
            }
            ImGui::SetCursorPosX(CursorPosXGroup);
            if(ImGui::Button("Clear", ButtonSize))
            {
                ClearBuffer();
            }
        ImGui::EndGroup();

        const ImVec2 canvasPos = ImGui::GetCursorScreenPos();
        const ImVec2 canvasSize = ImGui::GetContentRegionAvail();

        ImGui::PushClipRect(canvasPos, canvasPos + canvasSize, false);

        ImDrawList* WindowDrawList = ImGui::GetWindowDrawList();
        WindowDrawList->AddRectFilled(
            canvasPos,
            ImVec2(canvasPos.x + canvasSize.x, canvasPos.y + canvasSize.y),
            ImGui::ColorConvertFloat4ToU32({0.2f, 0.2f, 0.2f, 1.0f})
        );

        Controller->Render(static_cast<long>(DeltaTime * 1000.0f), canvasPos, canvasSize); //convert to millis

        ImGui::PopClipRect();
    }
    ImGui::End();

    if(DisplayDemoWindow) ImGui::ShowDemoWindow(&DisplayDemoWindow);
    return true;
}

void MainWindow::CopyTextToClipboard(const std::string& text) const
{
Controller->CopyToClipboard(text);
}

void MainWindow::ClearBuffer() const
{
Controller->GetBufferRef()->clear();
}

ImFont* MainWindow::LoadFonts(float pixel_size)
{
    // Just use the default font
    std::cout << "Using default font to avoid font loading issues." << std::endl;
    return ImGui::GetIO().Fonts->AddFontDefault();
}
