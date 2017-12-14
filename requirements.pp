package {'graphviz': ensure => present }
package {'libxml2': ensure => present }
#package {'texlive-bundledoc': ensure => present }

#package {'texlive-scheme-full': ensure => present }

package {'texlive-scheme-basic': ensure => present }
# for fullpage.sty
# TODO V Fix this: package {'texlive-preprint-svn30447.2011-36': ensure => present }

package {'perl-Text-Template': ensure => present }
package {'perl-XML-LibXML': ensure => present }

# For plantuml.
package {'java-1.8.0-openjdk-headless': ensure => present }
# Get plantuml at: https://sourceforge.net/projects/plantuml/files/plantuml.jar/download

# TODO rackdiag + nwdiag
