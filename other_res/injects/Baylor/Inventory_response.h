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
#ifndef edgeloader_h
#define edgeloader_h

#include "picture.h"


class EdgeLoader {
    private:
        Picture **edges = nullptr;
    
    public:
        int numEdges;
        /*
        * description: default constructor for EdgeLoader    
        * return: none                                           
        * precondition: none                             
        * postcondition: 9 edges are loaded in for puzzle pieces to have                  
        *                                                        
        */
        EdgeLoader();

        /*
        * description: copy constructor for EdgeLoader    
        * return: none                                           
        * precondition: a valid EdgeLoader other object exists                             
        * postcondition: 9 edges are loaded in from existing EdgeLoader
        *               for puzzle pieces to have                                                                        
        */
        EdgeLoader(const EdgeLoader &other);

        /*
        * description: assignment operator overload for EdgeLoader    
        * return: EdgeLoader object reference                                        
        * precondition: a valid EdgeLoader other object exists                             
        * postcondition: 9 edges are loaded in for puzzle pieces to have 
        *                from existing EdgeLoader                                                                         
        */
        EdgeLoader &operator=(const EdgeLoader &other);

        /*
        * description: assignment operator overload for EdgeLoader    
        * return: EdgeLoader object reference                                        
        * precondition: none                             
        * postcondition: 9 edges are loaded in for puzzle pieces to have 
        *                from existing EdgeLoader                                                                         
        */
        EdgeLoader &operator=(EdgeLoader &&other);

        /*
        * description: destructor for EdgeLoader    
        * return: none                                           
        * precondition: none                             
        * postcondition: memory associated with EdgeLoader is freed                                                                         
        */
        ~EdgeLoader();

        /*
        * description: get random edge for a puzzle piece    
        * return: Picture pointer                                           
        * precondition: EdgeLoader object exists                             
        * postcondition: a random edge is returned, and isFlipped (original edge or inverse)
        *               is randomly assigned T/F                                                                         
        */
        Picture *getRandomEdge(bool &isFlipped);
};

#endif // edgeloader.h
