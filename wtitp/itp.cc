/*
 * itp.cc
 *
 *  Created on: Nov 13, 2015
 */


#include <Wt/WApplication>

#include "IdeaToProduct.h"


Wt::WApplication *createApplication(const Wt::WEnvironment& env)
{
    return new IdeaToProduct(env);
}


int main(int argc, char **argv)
{
    return Wt::WRun(argc, argv, &createApplication);
}
