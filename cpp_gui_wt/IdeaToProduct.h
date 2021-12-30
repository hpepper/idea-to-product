/*
 * IdeaToProduct.h
 *
 *  Created on: Nov 13, 2015
 *      Author: cadm
 */

#ifndef IDEATOPRODUCT_H_
#define IDEATOPRODUCT_H_

#include <Wt/WApplication>

class IdeaToProduct : public Wt::WApplication  {
public:
	IdeaToProduct(const Wt::WEnvironment& env);
	virtual ~IdeaToProduct();
};

#endif /* IDEATOPRODUCT_H_ */
