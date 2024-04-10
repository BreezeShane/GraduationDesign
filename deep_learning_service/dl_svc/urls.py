from django.urls import path
from dl_svc import apis
 
urlpatterns = [
    path('train/', apis.train),
    path('test/', apis.test),
    path('validate/', apis.validate),
    path('inference/', apis.inference),
    path('show/', apis.training_show),
    path('prune/', apis.prune),
    path('compile/', apis.compile_model),
    path('harvest/', apis.get_compiled_module),
]