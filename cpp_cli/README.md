# 

Testing:
  make itp && ./itp Documentation/itp_itpp.xml && pdflatex test.latex && evince test.pdf

  enable core dumps: ulimit -c unlimited
  


## References
* TinyXml 2
  * http://www.grinninglizard.com/tinyxml2docs/index.html
  * https://github.com/leethomason/tinyxml2
  * http://www.grinninglizard.com/tinyxml2/
* http://www.cplusplus.com/reference/string/



Maybe I should start out by simply manipulating the XML structure and not load the XML structure into Classes.

# TODO

# Concept

## Concept for
* Goal: Generate the SW Architecture Document.
  * Initially  
  
* Load the whole structure
* Generate each of the sections of the SWArcDoc:
   * Title page.
   * Table of content.
   * Introduction
   * System overview
   * Mapping between views
   * Directory
   * Rationale, Background, and Design constraints
   * Module
     * Module decomposition
     * Module uses
     * Module generalization
     * Module layered
   * C&C
     * C&C comunicating-processes
     * C&C shared-data
     * 
   * Allocation
     * Allocation deployment
     * Allocation implementation
     * Allocation work assignment
   * Appendix
     * Requirement captures
     * Open issues/questions
     * Design decisions

