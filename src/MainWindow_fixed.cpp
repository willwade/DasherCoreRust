#include "MainWindow.h"

#include <filesystem>
#include <iostream>

MainWindow::MainWindow()
{
	ImGui::SetCurrentFont(LoadFonts(22.0f));

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
    ImVector<ImWchar> ranges;
    ImFontGlyphRangesBuilder builder;
    builder.AddRanges(ImGui::GetIO().Fonts->GetGlyphRangesJapanese());
    builder.AddText("\xE2\x96\xA1"); // box
    builder.BuildRanges(&ranges);

    ImGuiIO& io = ImGui::GetIO();
    
    // Try different paths for the main font
    const char* fontPaths[] = {
        "Resources/NotoSans-Medium.ttf",
        "../Resources/NotoSans-Medium.ttf",
        "../../Resources/NotoSans-Medium.ttf",
        "../../../Resources/NotoSans-Medium.ttf",
        "./Resources/NotoSans-Medium.ttf",
        "/Users/willwade/GitHub/DasherCoreRust/DasherUI-main/Resources/NotoSans-Medium.ttf"
    };
    
    ImFont* font = nullptr;
    for (const char* path : fontPaths) {
        std::cout << "Trying to load font from: " << path << std::endl;
        if (std::filesystem::exists(path)) {
            std::cout << "Font file exists at: " << path << std::endl;
            font = io.Fonts->AddFontFromFileTTF(path, pixel_size, nullptr, io.Fonts->GetGlyphRangesDefault());
            if (font != nullptr) {
                std::cout << "Successfully loaded font from: " << path << std::endl;
                break;
            } else {
                std::cout << "Failed to load font from: " << path << std::endl;
            }
        } else {
            std::cout << "Font file does not exist at: " << path << std::endl;
        }
    }
    
    if (font == nullptr) {
        std::cout << "Failed to load any font. Using default font." << std::endl;
        font = io.Fonts->AddFontDefault();
        return font;
    }
    
    // Try different paths for the Japanese font
    const char* jpFontPaths[] = {
        "Resources/NotoSansJP-Medium.otf",
        "../Resources/NotoSansJP-Medium.otf",
        "../../Resources/NotoSansJP-Medium.otf",
        "../../../Resources/NotoSansJP-Medium.otf",
        "./Resources/NotoSansJP-Medium.otf",
        "/Users/willwade/GitHub/DasherCoreRust/DasherUI-main/Resources/NotoSansJP-Medium.otf"
    };
    
    ImFontConfig config;
    config.MergeMode = true;
    
    bool jpFontLoaded = false;
    for (const char* path : jpFontPaths) {
        std::cout << "Trying to load Japanese font from: " << path << std::endl;
        if (std::filesystem::exists(path)) {
            std::cout << "Japanese font file exists at: " << path << std::endl;
            if (io.Fonts->AddFontFromFileTTF(path, pixel_size, &config, ranges.Data) != nullptr) {
                std::cout << "Successfully loaded Japanese font from: " << path << std::endl;
                jpFontLoaded = true;
                break;
            } else {
                std::cout << "Failed to load Japanese font from: " << path << std::endl;
            }
        } else {
            std::cout << "Japanese font file does not exist at: " << path << std::endl;
        }
    }
    
    if (!jpFontLoaded) {
        std::cout << "Failed to load Japanese font." << std::endl;
    }
    
    io.Fonts->Build();
    return font;
}
