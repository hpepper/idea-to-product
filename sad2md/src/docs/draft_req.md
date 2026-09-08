# Draft requirements

- Must be able to convert itp.xml files to SwArcDoc markdown files.
- Must support multiple .xml as input.
- Must support context diagram generation
- Must support MSC generation
- DIAG-FR-008 - Must support deployment diagram generation.
  - Generate graphically
  - Generate textual - alphabetical bullet list.
- XML-FR-008 - Use common xml structure to define diagrams, for both textual and graphical representation.
  - TODO - move this to a common .md since it covers all applications.
  - See #### ComponentRelation - ConnectionType in the ../README.md


#### DIAG-FR-008 - design

I order to generate the diagram all components that are contained by ComponentAlpha, must be listed under the definition of ComponentAlpha

Well-Actually; it looks like I can just add them in any order, and mermaid will handle it, I wonder if gitlab can also render it correctly?
The layout changes though.

Make the textual representation just an usorted list ordered alphabetically.


```mermaid
graph TB
  subgraph VM1 [Game VM]
    subgraph CS2[CS2 server]
      configuration
      game
    end
    Filebeat
  end

  subgraph VM2 [Statistics VM]
    HAProxy
    subgraph Docker
      Logstash
      Elasticsearch
      Kibana
      Statistics2web
      WebServer
    end
  end
```

Un-ordered presentation


```mermaid
graph TB
  subgraph VM1 [Game VM]
    CS2[CS2 server]
  end

  subgraph VM1 [Game VM]
    Filebeat
  end

  subgraph CS2
    configuration
    game    
  end


  subgraph VM2 [Statistics VM]
    HAProxy
    Docker
  end

  subgraph Docker
      Logstash
      Elasticsearch
      Kibana
      Statistics2web
      WebServer
  end

  Logstash---Elasticsearch
  Elasticsearch---Kibana
  HAProxy---Kibana
  HAProxy---Logstash
  HAProxy---WebServer
  Elasticsearch---Statistics2web
  Statistics2web---WebServer
```
