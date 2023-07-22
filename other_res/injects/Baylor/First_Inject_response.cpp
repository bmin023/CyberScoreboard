/* Author: Brendon Kofink
 *         Wesley Goyette
 *         Aaron Sierra
 *         David Day
 *         Lucy Ray
 * Assignment Title: Epicer Puzzle
 * Assignment Description: A Puzzle game where the user can
 *   solve a puzzle by moving and snapping pieces into place.
 * Due Date: 5/4/2023
 * Date Created: 4/20/2023
 * Date Last Modified: 4/20/2023
 */
#include "SDL_Plotter.h"
#include "Typer.h"
#include "picture.h"
#include "piece.h"
#include "puzzle.h"
#include "vec2.h"
#include <cmath>
#include <fstream>
#include <iostream>
#include <string>

using namespace std;

// Data Abstraction:
// 		A puzzle game is initialized to have TITLE state
//      Strings with the Picture file paths are initialized
// 		A SDL Plotter object is initialized, and its associated Drawer object is 
//          initialized with the window
//      A Puzzle object is declared
//      Point offset is declared
//      A Typer object is declared, to write text to the screen
//
// Input: 
// 		The program responds to SDL Plotter events such as keyboard hits and 
//      mouse clicks
// Process:
// 		While the game state is TITLE, a menu is displayed
//          When the user selects '1', '2', or '3', the associated puzzle is 
//          initialized and the screen is cleared.
//          The game is now in PLAY state
// 		The player can pick up puzzle pieces by clicking on them, and place them 
//      by clicking again. 
//      If a piece is placed close enough to its correct neighbor, it "snaps" 
//      together, and the pieces move as one
//      If the selected piece is connected to all other puzzle pieces, the user 
//      has won. The game state is now WIN
//      The user can play again by hitting the space key, which will change the 
//      game state to TITLE
// Output: 
// 		The puzzle game is displayed for the user to play and interact with
// Assumptions: 
// 		It is assumed that the user only wants to play one puzzle at a time
// 		It is assumed that all pictures and edges have been processed by the 
//      "convert.py" script and are in 
//      "pictureTXTs" directory
//

/*
* description: move Piece    
* return: void                                           
* precondition: There is a valid puzzle piece, the point parameter is a point
*               on screen. Drawer has been initialized                             
* postcondition: The puzzle piece has been moved and drawn to the appropriate
*                location in the window                                                                        
*/
void movePiece(Piece &piece, point p, Drawer &drawer);

/*
* description: test if a piece can be snapped with a neighbor piece 
* return: boolean true if it can be connected to its neighbor, false if not                                           
* precondition: Puzzle pieces are valid                             
* postcondition: a boolean is returned, true if a piece can snap with it's
*               neighbor in proximity, false if not                                                                          
*/
bool testSnappable(Piece &piece, Drawer &drawer);

int main(int argc, char **argv) {
    enum gameState {
        TITLE,
        PLAY,
        WIN,
    };
    gameState state = TITLE;

    string colorPNG = "./picturetxts/colors.jpg.txt";
    string carPNG = "./picturetxts/car.jpg.txt";
    string finePNG = "./picturetxts/fine.jpeg.txt";
    Piece *selectedPiece = nullptr;
    SDL_Plotter window(1000, 1000, true);
    Drawer drawer = Drawer(window);
    Puzzle puzzle = Puzzle(colorPNG);
    point offset;
    Typer t;

    window.initSound("Click.wav");
    window.initSound("GoodDesign.wav");

    while (!window.getQuit()) {
        while (state == TITLE && !window.getQuit()) {
            t.Write("EPIC PUZZLE GAME", window, Vec2(100, 160), color(0, 0, 0),
                    5, false);
            t.Write("Djisktras Disciples", window, Vec2(100, 220),
                    color(0, 0, 0), 3, false);
            t.Write("Select your puzzle", window, Vec2(100, 300), 
                    color(255, 0, 0),7, false);
            t.Write("1 - colors", window, Vec2(100, 360), 
                    color(0, 0, 0), 3, false);
            t.Write("2 - car", window, Vec2(100, 380),
                    color(0, 0, 0), 3, false);
            t.Write("3 - everything is fine", window, Vec2(100, 400),
                    color(0, 0, 0), 3, false);

            if(window.kbhit()){
                char choice = window.getKey();
                switch (choice){
                    case '1':
                        state = PLAY;
                        window.clear();
                        puzzle = Puzzle(colorPNG);
                        break;
                    case '2':
                        state = PLAY;
                        window.clear();
                        puzzle = Puzzle(carPNG);
                        break;
                    case '3':
                        state = PLAY;
                        window.clear();
                        puzzle = Puzzle(finePNG);
                        break;
                }
            }
            window.update();
        }
        if (window.mouseClick()) {
            window.getMouseClick();
        }
        while (state != TITLE && !window.getQuit()) {
            if (selectedPiece != nullptr) {
                point p;
                window.getMouseLocation(p.x, p.y);
                p.x -= offset.x;
                p.y -= offset.y;
                movePiece(*selectedPiece, p, drawer);
            }
            if (window.mouseClick()) {
                point click = window.getMouseClick();
                if (selectedPiece != nullptr) {
                    if (testSnappable(*selectedPiece, drawer)) {
                        window.playSound("Click.wav");
                    }
                    if (state == PLAY && selectedPiece->getConnected() == puzzle.pieces()) {
                        state = WIN;
                        window.playSound("GoodDesign.wav");
                        if (window.kbhit()) {
                            window.getKey();
                        }
                    }
                    selectedPiece = nullptr;
                } else if (puzzle.mouseClick(click, &selectedPiece)) {
                    offset.x = click.x - selectedPiece->getPos().x;
                    offset.y = click.y - selectedPiece->getPos().y;
                } // check for mouse click on puzzle
                while (window.mouseClick()) {
                    window.getMouseClick();
                }
            }
            puzzle.draw(drawer);

            if (state == WIN) {
                t.Write("You won", window, Vec2(100, 160), color(0, 0, 0), 10,
                        false);
                t.Write("press space to", window, Vec2(100, 300),
                        color(255, 0, 0), 7, false);
                t.Write("play again", window, Vec2(100, 340),
                        color(255, 0, 0), 7, false);

                if(window.kbhit()) {
                    if(window.getKey() == ' '){
                        state = TITLE;
                        window.clear();
                    }
                }
            }

            window.update();
        }
    }
    return 0;
}


bool testSnappable(Piece &piece, Drawer &drawer) {
    int pieceNum = piece.getConnected();
    if (piece.isSnappable(NORMAL)) {
        movePiece(piece, piece.snap(NORMAL).toPoint(), drawer);
        piece.connect(NORMAL);
    } 
    if (piece.isSnappable(RIGHT)) {
        movePiece(piece, piece.snap(RIGHT).toPoint(), drawer);
        piece.connect(RIGHT);
    } 
    if (piece.isSnappable(FLIPPED)) {
        movePiece(piece, piece.snap(FLIPPED).toPoint(), drawer);
        piece.connect(FLIPPED);
    }
    if (piece.isSnappable(LEFT)) {
        movePiece(piece, piece.snap(LEFT).toPoint(), drawer);
        piece.connect(LEFT);
    } 
    
    return piece.getConnected() > pieceNum;
}

void movePiece(Piece &selectedPiece, point p, Drawer &drawer) {
    Background bg = Background();
    selectedPiece.move(Vec2::fromPoint(p), drawer, bg);
    selectedPiece.setMoved(false);
}
