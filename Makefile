
all: sad.pdf 


distclean: clean
	-rm *.pdf
	- rm *.tex
	- rm *.log
	- rm *.gv
	- rm *.ps
	- rm *.lof
	- rm *.lot
	- rm sub_makefile.mk


clean:
	-rm *.dvi
	-rm *.png
	-rm *.aux
	-rm *.toc
	-rm *~


sad.pdf: sad.dvi
	dvipdf sad.dvi

sad.dvi: sad.tex
	latex sad.tex
	latex sad.tex
	latex sad.tex

sad.tex: itp.xml ../idea-to-product/itp2doc.pl ../idea-to-product/Graphviz_CnCDiagram.tmpl ../idea-to-product/Graphviz_componentDiagram.tmpl ../idea-to-product/Graphviz_contextModel.tmpl ../idea-to-product/LaTeX_article_project_executive_summary.tmpl ../idea-to-product/LaTeX_componentDiagram.tmpl ../idea-to-product/LaTeX_fullCharter.tmpl ../idea-to-product/LaTeX_ViewPacket.tmpl ../idea-to-product/LaTeX_ProjectExecutiveSummary.tmpl ../idea-to-product/LaTeX_sipoc.tmpl ../idea-to-product/LaTeX_SwArcDoc.tmpl not_present.tex
	../idea-to-product/itp2doc.pl SoftwareArchitectureDocumentation 1
	-make -f sub_makefile.mk all

ExecutiveSummary: itp.xml latex_project_executive_summary.pdf

latex_project_executive_summary.pdf: latex_project_executive_summary.dvi itp.xml
	dvipdf latex_project_executive_summary.dvi
	
latex_project_executive_summary.dvi: itp.xml ProjectExecutiveCharter_1.tex
	latex latex_project_executive_summary.tex
	latex latex_project_executive_summary.tex
	latex latex_project_executive_summary.tex

ProjectExecutiveCharter_1.tex: itp.xml
	../idea-to-product/itp2doc.pl ProjectExecutiveSummary 1

not_present.tex:
	echo "Not present." > not_present.tex
