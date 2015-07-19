
all: sad.pdf 


distclean: clean
	-rm *.pdf
	- rm *.tex
	- rm *.log
	- rm *.gv
	- rm *.ps
	- rm *.lof
	- rm *.lot


clean:
	-rm *.dvi
	-rm *.png
	-rm *.aux
	-rm *.toc


sad.pdf: sad.dvi
	dvipdf sad.dvi

sad.dvi: sad.tex
	latex sad.tex
	latex sad.tex
	latex sad.tex

sad.tex: itp.xml not_present.tex
	./itp2doc.pl SoftwareArchitectureDocumentation 1
	-make -f sub_makefile.mk all

ExecutiveSummary: itp.xml latex_project_executive_summary.pdf

latex_project_executive_summary.pdf: latex_project_executive_summary.dvi itp.xml
	dvipdf latex_project_executive_summary.dvi
	
latex_project_executive_summary.dvi: itp.xml ProjectExecutiveCharter_1.tex
	latex latex_project_executive_summary.tex
	latex latex_project_executive_summary.tex
	latex latex_project_executive_summary.tex

ProjectExecutiveCharter_1.tex: itp.xml
	./itp2doc.pl ProjectExecutiveSummary 1

not_present.tex:
	echo "Not present." > not_present.tex
